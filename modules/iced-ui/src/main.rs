#[macro_use]
extern crate tracing;

use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting OSC server...");

    let http = avisaver_osc::queryserver::QueryServer::start().await.unwrap();
    let zeroconf = avisaver_osc::zeroconf::ZeroconfServer::start(http.port()).unwrap();

    time::sleep(Duration::from_secs(10)).await;

    info!("Stopping OSC server...");

    zeroconf.stop().await;
    http.stop().await;
}
