use crate::error::OscError;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tokio::sync::Notify;
use zeroconf::prelude::*;
use zeroconf::{MdnsService, ServiceRegistration, ServiceType};

pub struct ZeroconfServer {
    running: Arc<AtomicBool>,
    shutdown: Arc<Notify>,
}

impl ZeroconfServer {
    pub fn start(osc_port: u16) -> Result<ZeroconfServer, OscError> {
        info!("Starting Zeroconf server for port {}", osc_port);
        let mut service = MdnsService::new(ServiceType::new("oscjson", "tcp")?, osc_port);
        service.set_name("avisaver");
        service.set_registered_callback(Box::new(zeroconf_service_register));
        let event_loop = service.register()?;
        let running = Arc::new(AtomicBool::new(true));
        let running1 = running.clone();
        let shutdown = Arc::new(Notify::new());
        let shutdown1 = shutdown.clone();
        thread::spawn(move || {
            info!("Zeroconf server started.");
            let _ = &service;
            while running1.load(Ordering::Acquire) {
                match event_loop.poll(Duration::from_secs(1)) {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("Error polling zeroconf server: {:?}", err);
                    }
                }
            }
            info!("Zeroconf server stopped.");
            shutdown1.notify_one();
        });
        Ok(ZeroconfServer { running, shutdown })
    }

    pub async fn stop(&self) {
        if self.running.swap(false, Ordering::AcqRel) {
            info!("Stopping Zeroconf server...");
            self.shutdown.notified().await;
        }
    }
}

impl Drop for ZeroconfServer {
    fn drop(&mut self) {
        if self.running.swap(false, Ordering::AcqRel) {
            info!("Stopping Zeroconf server...");
        }
    }
}

fn zeroconf_service_register(
    reg: Result<ServiceRegistration, zeroconf::error::Error>,
    _ctx: Option<Arc<dyn Any + Send + Sync>>,
) {
    match reg {
        Ok(service) => {
            info!("Service registered: {:?}", service);
        }
        Err(err) => {
            warn!("Service registration error: {:?}", err);
        }
    }
}
