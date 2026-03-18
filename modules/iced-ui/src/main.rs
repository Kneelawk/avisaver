#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting OSC server...");

    let opts = avisaver_osc::QueryOptions {
        app_name: "avisaver".to_string(),
        directories: vec!["/avatar".to_string()],
    };

    let http = avisaver_osc::queryserver::QueryServer::start(&opts)
        .await
        .unwrap();
    let zeroconf = avisaver_osc::zeroconf::ZeroconfServer::start(http.port(), &opts).unwrap();

    tokio::signal::ctrl_c().await.unwrap();

    info!("Stopping OSC server...");

    zeroconf.stop().await;
    http.stop().await;
}
