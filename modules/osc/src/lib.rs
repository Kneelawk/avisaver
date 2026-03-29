#[macro_use]
extern crate tracing;

use crate::error::{OSCShutdownError, OSCStartupError};
use crate::format::{OSCQHostInfo, OSCQNode};
use crate::queryserver::QueryServer;
use crate::zeroconf::ZeroconfServer;
use rosc::OscPacket;
use std::future::ready;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Notify;

pub mod error;
pub mod format;
pub mod queryserver;
pub mod zeroconf;

/// Listener for various OSC events.
#[allow(unused_variables)]
pub trait OSCListener {
    /// Listen for when a packet is received.
    fn packet_received(&self, from: SocketAddr, packet: OscPacket) -> impl Future + Send + Sync {
        ready(())
    }
}

/// Options to describe which OSC parameters to accept
#[derive(Debug, Default, Clone)]
pub struct QueryOptions<T: OSCListener + Send + Sync + 'static> {
    /// The name of the OSCQuery application
    pub app_name: String,
    /// A set of all the directories to advertise that we listen to.
    pub directories: Vec<String>,
    /// The listener for OSC events
    pub listener: T,
}

/// OSCQuery application
pub struct OSCQuery {
    http: QueryServer,
    zeroconf: ZeroconfServer,
    stop: Arc<Notify>,
    stopped: Arc<Notify>,
}

impl OSCQuery {
    pub async fn new<T: OSCListener + Send + Sync + 'static>(
        opts: QueryOptions<T>,
    ) -> Result<OSCQuery, OSCStartupError> {
        let QueryOptions {
            app_name,
            directories,
            listener,
        } = opts;

        let listener = Arc::new(listener);

        let api_app_name = format!("{}-{:X}", &app_name, rand::random::<u32>());

        info!("Binding UDP listener...");
        let sock = UdpSocket::bind(("0.0.0.0", 0)).await?;
        let udp_addr = sock.local_addr()?;

        info!("UDP listener bound to port {}", udp_addr.port());

        let host_info = OSCQHostInfo {
            name: Some(app_name.clone()),
            osc_ip: Some("127.0.0.1".to_string()),
            osc_port: Some(udp_addr.port()),
            ..Default::default()
        };

        // build node structure
        let mut root = Default::default();
        for dir in &directories {
            insert_path(&mut root, dir);
        }

        let http = QueryServer::start(host_info, root).await?;

        let zeroconf = ZeroconfServer::start(http.port(), &api_app_name)?;

        let stop = Arc::new(Notify::new());
        let stop1 = stop.clone();
        let stopped = Arc::new(Notify::new());
        let stopped1 = stopped.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; rosc::decoder::MTU];

            loop {
                tokio::select! {
                    _ = stop1.notified() => {
                        break;
                    }
                    recv = sock.recv_from(&mut buf) => {
                        match recv {
                            Ok((size, addr)) => {
                                trace!("Received packet of size {size} from {addr}");
                                match rosc::decoder::decode_udp(&buf) {
                                    Ok((_, packet)) => {
                                        listener.packet_received(addr, packet).await;
                                    }
                                    Err(err) => {
                                        warn!("Error decoding OSC packet: {err:?}");
                                    }
                                }
                            }
                            Err(err) => {
                                warn!("Error reading UDP datagram: {err:?}");
                            }
                        }
                    }
                }
            }

            stopped1.notify_waiters();
        });

        Ok(OSCQuery {
            http,
            zeroconf,
            stop,
            stopped,
        })
    }

    pub async fn shutdown(&mut self) -> Result<(), OSCShutdownError> {
        self.stop.notify_one();
        self.stopped.notified().await;
        self.zeroconf.shutdown()?;
        self.http.shutdown().await;
        Ok(())
    }
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
