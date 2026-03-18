use crate::error::OscError;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::server::graceful::GracefulShutdown;
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Notify;

pub struct QueryServer {
    running: Arc<AtomicBool>,
    shutdown: Arc<Notify>,
    complete: Arc<Notify>,
    port: u16,
}

impl QueryServer {
    pub async fn start() -> Result<QueryServer, OscError> {
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

                        let conn = server.serve_connection_with_upgrades(stream, service_fn(query_service));
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

    pub fn port(&self) -> u16 {
        self.port
    }

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

async fn query_service(_: Request<Incoming>) -> Result<Response<String>, Infallible> {
    Ok(Response::new("Hello World!".to_string()))
}
