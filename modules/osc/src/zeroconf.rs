use crate::QueryOptions;
use crate::error::OscError;
use simple_mdns::async_discovery::ServiceDiscovery;
use simple_mdns::{InstanceInformation, SimpleMdnsError};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
// use zeroconf::prelude::*;
// use zeroconf::{MdnsService, ServiceRegistration, ServiceType, TxtRecord};

/// Zeroconf server that supports the mDNS lookup portion of OSCQuery queries.
pub struct ZeroconfServer {
    running: Arc<AtomicBool>,
    shutdown: Arc<Notify>,
    discovery: Arc<Mutex<ServiceDiscovery>>,
}

impl ZeroconfServer {
    /// Start a zeroconf server on a given port.
    ///
    /// See [`crate::queryserver::QueryServer::port`]
    pub fn start(
        osc_port: u16,
        name: &str,
        protocol: &str,
        opts: &QueryOptions,
    ) -> Result<ZeroconfServer, OscError> {
        info!("Starting Zeroconf server for port {}", osc_port);
        let discovery = Arc::new(Mutex::new(ServiceDiscovery::new(
            InstanceInformation::new(opts.app_name.clone())
                .with_ip_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
                .with_port(osc_port)
                .with_attribute("txtvers".to_string(), Some("1".to_string())),
            &format!("_{}._{}.local", name, protocol),
            60,
        )?));
        let discovery1 = discovery.clone();

        // let mut service = MdnsService::new(ServiceType::new(name, protocol)?, osc_port);
        // service.set_name(&opts.app_name);
        // service.set_registered_callback(Box::new(zeroconf_service_register));
        //
        // let mut txt = TxtRecord::new();
        // txt.insert("txtvers", "1")?;
        // service.set_txt_record(txt);

        // service.set_host(&format!("{}.{name}.local", &opts.app_name));
        // service.set_domain("127.0.0.1");
        // service.set_host("127.0.0.1");
        // service.set_network_interface(NetworkInterface::)

        // let event_loop = service.register()?;
        let running = Arc::new(AtomicBool::new(true));
        let running1 = running.clone();
        let shutdown = Arc::new(Notify::new());
        let shutdown1 = shutdown.clone();

        tokio::spawn(async move {
            while running1.load(Ordering::Acquire) {
                tokio::time::sleep(Duration::from_secs(5)).await;
                match discovery1.lock().await.announce(false).await {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("Error announcing mdns service: {err:?}");
                    }
                }
            }
            shutdown1.notify_waiters();
        });

        // thread::spawn(move || {
        //     info!("Zeroconf server started.");
        //     let _ = &service;
        //     while running1.load(Ordering::Acquire) {
        //         match event_loop.poll(Duration::from_secs(1)) {
        //             Ok(_) => {}
        //             Err(err) => {
        //                 warn!("Error polling zeroconf server: {:?}", err);
        //             }
        //         }
        //     }
        //     info!("Zeroconf server stopped.");
        //     shutdown1.notify_one();
        // });
        Ok(ZeroconfServer {
            running,
            shutdown,
            discovery,
        })
    }

    /// Stop the zeroconf server and wait for it to shut down.
    ///
    /// Dropping the server also stops it but does not wait for it to shut down.
    pub async fn stop(&mut self) {
        if self.running.swap(false, Ordering::AcqRel) {
            info!("Stopping Zeroconf server...");
            self.shutdown.notified().await;
        }
        self.discovery
            .lock()
            .await
            .remove_service_from_discovery()
            .await;
    }
}

impl Drop for ZeroconfServer {
    fn drop(&mut self) {
        if self.running.swap(false, Ordering::AcqRel) {
            info!("Stopping Zeroconf server...");
        }
    }
}

// fn zeroconf_service_register(
//     reg: Result<ServiceRegistration, zeroconf::error::Error>,
//     _ctx: Option<Arc<dyn Any + Send + Sync>>,
// ) {
//     match reg {
//         Ok(service) => {
//             info!("Service registered: {:?}", service);
//         }
//         Err(err) => {
//             warn!("Service registration error: {:?}", err);
//         }
//     }
// }
