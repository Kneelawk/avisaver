use crate::error::OscError;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsService, ServiceRegistration, ServiceType};

pub struct ZeroconfServer {
    running: Arc<AtomicBool>,
}

impl ZeroconfServer {
    pub fn start(osc_port: u16) -> Result<ZeroconfServer, OscError> {
        let mut service = MdnsService::new(ServiceType::new("oscjson", "tcp")?, osc_port);
        service.set_name("avisaver");
        service.set_registered_callback(Box::new(zeroconf_service_register));
        let event_loop = service.register()?;
        let running = Arc::new(AtomicBool::new(true));
        let running1 = running.clone();
        thread::spawn(move || {
            let _ = &service;
            while running1.load(Ordering::Acquire) {
                match event_loop.poll(Duration::from_secs(1)) {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("Error polling zeroconf server: {:?}", err);
                    }
                }
            }
        });
        Ok(ZeroconfServer { running })
    }
}

impl Drop for ZeroconfServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
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
