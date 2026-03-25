use crate::QueryOptions;
use crate::error::OscError;
use crate::format::{OSCQHostInfo, OSCQNode};
use hyper::body::Incoming;
use hyper::http::HeaderValue;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::server::graceful::GracefulShutdown;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Notify;

/// Http server that supports the http-lookup portion of OSCQuery queries.
pub struct QueryServer {
    running: Arc<AtomicBool>,
    shutdown: Arc<Notify>,
    complete: Arc<Notify>,
    port: u16,
}

impl QueryServer {
    /// Start the http server.
    pub async fn start(opts: &QueryOptions) -> Result<QueryServer, OscError> {
        let opts = Arc::new(opts.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let server = auto::Builder::new(TokioExecutor::new());
        let graceful = GracefulShutdown::new();

        let running = Arc::new(AtomicBool::new(true));
        let shutdown = Arc::new(Notify::new());
        let shutdown1 = shutdown.clone();
        let complete = Arc::new(Notify::new());
        let complete1 = complete.clone();

        info!("Starting OSCQuery HTTP server on port: {}", port);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    conn = listener.accept() => {
                        let (stream, client_addr) = match conn {
                            Ok(conn) => conn,
                            Err(err) => {
                                warn!("Error accepting new incoming http connection: {:?}", err);
                                continue;
                            }
                        };

                        info!("Received HTTP connection from: {}", client_addr);

                        let stream = TokioIo::new(stream);

                        let opts1 = opts.clone();
                        let conn = server.serve_connection_with_upgrades(stream, service_fn(move |req| {
                            let opts2 = opts1.clone();
                            async move {
                                query_service(req, opts2.clone()).await
                            }
                        }));
                        let conn = graceful.watch(conn.into_owned());

                        tokio::spawn(async move {
                            if let Err(e) = conn.await {
                                warn!("HTTP connection error: {:?}", e);
                            }
                        });
                    },

                    _ = shutdown1.notified() => {
                        info!("HTTP Server shutdown signal received, shutting down HTTP server...");
                        drop(listener);
                        break;
                    }
                }
            }

            tokio::select! {
                _ = graceful.shutdown() => {
                    info!("HTTP server successfully shutdown.");
                }
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    error!("HTTP server did not shutdown within the allotted time. Killing HTTP server...");
                }
            }

            complete1.notify_waiters();
        });

        Ok(QueryServer {
            running,
            shutdown,
            complete,
            port,
        })
    }

    /// The port that the server bound to.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Stop the http server and wait for it to finish.
    ///
    /// Note: dropping the query server will also request the server to stop but does not wait for
    /// the server to finish.
    pub async fn stop(&self) {
        if self.running.swap(false, Ordering::AcqRel) {
            info!("Stopping HTTP server...");
            self.shutdown.notify_one();
            self.complete.notified().await;
        }
    }
}

impl Drop for QueryServer {
    fn drop(&mut self) {
        if self.running.swap(false, Ordering::AcqRel) {
            info!("Stopping HTTP server...");
            self.shutdown.notify_one();
        }
    }
}

fn response<S, T, V, E>(status_code: S, content_type: V, text: T) -> Result<Response<String>, E>
where
    S: TryInto<StatusCode>,
    <S as TryInto<StatusCode>>::Error: Into<hyper::http::Error>,
    T: ToString,
    V: TryInto<HeaderValue>,
    <V as TryInto<HeaderValue>>::Error: Into<hyper::http::Error>,
{
    let str = text.to_string();
    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", content_type)
        .header("Content-Length", str.len())
        .body(str)
        .expect("bad response build"))
}

fn error<S: TryInto<StatusCode>, T: ToString, E>(
    status_code: S,
    text: T,
) -> Result<Response<String>, E>
where
    <S as TryInto<StatusCode>>::Error: Into<hyper::http::Error>,
{
    response(status_code, "text/plain", text)
}

fn json<T: Serialize, E>(obj: &T) -> Result<Response<String>, E> {
    match serde_json::to_string(obj) {
        Ok(str) => response(200, "application/json", str),
        Err(err) => {
            error!("Error serializing object to json: {:?}", err);
            error(500, "internal server error")
        }
    }
}

fn text<T: Display + ?Sized, E>(val: &T) -> Result<Response<String>, E> {
    response(200, "text/plain", val)
}

fn insert_path(root: &mut OSCQNode, path: &str) {
    let mut full_path = String::new();
    let mut node = root;
    for piece in path.split('/') {
        if piece.is_empty() {
            continue;
        }

        full_path += "/";
        full_path += piece;

        if !node.contents.contains_key(piece) {
            node.contents.insert(
                piece.to_string(),
                OSCQNode {
                    full_path: full_path.clone(),
                    ..Default::default()
                },
            );
        }

        node = node
            .contents
            .get_mut(piece)
            .expect("node get contents missing piece");
    }
}

async fn query_service(
    req: Request<Incoming>,
    opts: Arc<QueryOptions>,
) -> Result<Response<String>, Infallible> {
    let query = req.uri().query();
    if query.is_some_and(|s| s.starts_with("HOST_INFO")) {
        return json(&OSCQHostInfo {
            name: Some(opts.app_name.clone()),
            osc_port: Some(opts.udp_port),
            ..Default::default()
        });
    }

    // build node structure
    // FIXME: should we be doing ahead of time???
    let mut root = Default::default();
    for dir in &opts.directories {
        insert_path(&mut root, dir);
    }

    // find the node that's being requested
    let mut node = &root;
    let path = req.uri().path();
    for piece in path.split('/') {
        if piece.is_empty() {
            continue;
        }

        if node.contents.contains_key(piece) {
            node = node
                .contents
                .get(piece)
                .expect("node lookup contents missing");
        } else {
            return error(404, "method not found");
        }
    }

    if let Some(query) = query {
        if query.starts_with("FULL_PATH") {
            text(&node.full_path)
        } else if query.starts_with("TYPE") {
            if let Some(ty) = &node.ty {
                text(ty)
            } else {
                text("")
            }
        } else if query.starts_with("VALUE") {
            if let Some(value) = &node.value {
                json(value)
            } else {
                if node.access.is_some_and(|a| a & 1 == 0) {
                    response(204, "text/plain", "")
                } else {
                    response(200, "application/json", "{}")
                }
            }
        } else if query.starts_with("ACCESS") {
            if let Some(access) = node.access {
                text(&access)
            } else {
                text("0")
            }
        } else {
            error(400, "unsupported attribute query")
        }
    } else {
        json(node)
    }
}
