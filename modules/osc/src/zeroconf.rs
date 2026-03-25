use crate::QueryOptions;
use crate::error::OscError;
use searchlight::broadcast::{BroadcasterBuilder, BroadcasterHandle, ServiceBuilder};
use searchlight::net::IpVersion;
use std::net::{IpAddr, Ipv4Addr};

/// Zeroconf server that supports the mDNS lookup portion of OSCQuery queries.
pub struct ZeroconfServer {
    broadcaster: Option<BroadcasterHandle>,
}

impl ZeroconfServer {
    /// Start a zeroconf server on a given port.
    ///
    /// See [`crate::queryserver::QueryServer::port`]
    pub fn start(osc_port: u16, opts: &QueryOptions) -> Result<ZeroconfServer, OscError> {
        info!("Starting Zeroconf server for port {}", osc_port);
        let broadcaster = BroadcasterBuilder::new()
            .loopback()
            .add_service(
                ServiceBuilder::new("_oscjson._tcp.local.", &opts.app_name, osc_port)?
                    .add_ip_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
                    .add_txt("txtvers=1")
                    .build()?,
            )
            .build(IpVersion::V4)?
            .run_in_background();

        Ok(ZeroconfServer {
            broadcaster: Some(broadcaster),
        })
    }

    /// Stop the zeroconf server.
    ///
    /// Dropping the server also stops it.
    pub fn stop(&mut self) -> Result<(), OscError> {
        if let Some(broadcaster) = self.broadcaster.take() {
            broadcaster.shutdown()?;
        }

        Ok(())
    }
}

impl Drop for ZeroconfServer {
    fn drop(&mut self) {
        if let Err(err) = self.stop() {
            warn!("Error shutting down zeroconf server: {err:?}");
        }
    }
}
