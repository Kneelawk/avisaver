#[macro_use]
extern crate tracing;

use rand::RngExt;
use rosc::OscPacket;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting OSC server...");

    let mut rng = rand::rng();

    info!("Binding UPD listener...");
    let sock = UdpSocket::bind(("0.0.0.0", 0)).await.unwrap();
    let udp_addr = sock.local_addr().unwrap();

    let opts = avisaver_osc::QueryOptions {
        app_name: format!("avisaver-{:X}", rng.random::<u32>()),
        directories: vec!["/avatar".to_string()],
        udp_port: udp_addr.port(),
    };

    let http = avisaver_osc::queryserver::QueryServer::start(&opts)
        .await
        .unwrap();
    let mut zeroconf = avisaver_osc::zeroconf::ZeroconfServer::start(http.port(), &opts).unwrap();

    let mut buf = [0u8; rosc::decoder::MTU];
    loop {
        tokio::select! {
            recv = sock.recv_from(&mut buf) => {
                match recv {
                    Ok((size, addr)) => {
                        info!("Received packet with size {} from: {}", size, addr);
                        let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                        handle_packet(packet);
                    }
                    Err(err) => {
                        warn!("Error reading UDP datagram: {:?}", err);
                        break;
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    info!("Stopping OSC server...");

    zeroconf.stop().unwrap();
    http.stop().await;
}

fn handle_packet(packet: OscPacket) {
    info!("Received OSC Packet: {:?}", packet);
}
