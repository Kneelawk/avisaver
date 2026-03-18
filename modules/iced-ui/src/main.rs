#[macro_use]
extern crate tracing;

use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting OSC server...");

    let server = avisaver_osc::zeroconf::ZeroconfServer::start(25569).unwrap();

    time::sleep(Duration::from_secs(10)).await;

    info!("Stopping OSC server...");

    server.stop().await;
}
