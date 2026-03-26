#[macro_use]
extern crate tracing;

use avisaver_osc::{OSCListener, OSCQuery, QueryOptions};
use rosc::OscPacket;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting OSC server...");

    let mut osc = OSCQuery::new(QueryOptions {
        app_name: "avisaver".to_string(),
        directories: vec!["/avatar".to_string()],
        listener: MyListener,
    })
    .await
    .unwrap();

    tokio::signal::ctrl_c().await.unwrap();

    info!("Stopping OSC server...");

    osc.shutdown().await.unwrap();
}

struct MyListener;

#[allow(refining_impl_trait)]
impl OSCListener for MyListener {
    async fn packet_received(&self, packet: OscPacket) {
        info!("Received OSC Packet: {packet:?}");
    }
}
