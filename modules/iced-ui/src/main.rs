#[macro_use]
extern crate tracing;

// use agnostic_mdns::{ServerOptions, ServiceBuilder};
use rand::RngExt;
use rosc::OscPacket;
use searchlight::broadcast::{BroadcasterBuilder, ServiceBuilder};
use searchlight::net::IpVersion;
use std::net::{IpAddr, Ipv4Addr};
// use agnostic_mdns::tokio::Server;
use tokio::net::UdpSocket;
// use zeroconf::browser::TMdnsBrowser;

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
    // let mut zeroconf_udp =
    //     avisaver_osc::zeroconf::ZeroconfServer::start(udp_addr.port(), "osc", "udp", &opts)
    //         .unwrap();
    // let mut zeroconf_tcp =
    //     avisaver_osc::zeroconf::ZeroconfServer::start(http.port(), "oscjson", "tcp", &opts)
    //         .unwrap();

    // let service = ServiceBuilder::new(opts.app_name.as_str().into(), "_oscjson._tcp".into())
    //     .with_port(http.port())
    //     .with_ip(IpAddr::V4(Ipv4Addr::LOCALHOST))
    //     .finalize()
    //     .unwrap();
    //
    // let srv = Server::new(service, ServerOptions::default()).await.unwrap();

    let broadcaster = BroadcasterBuilder::new()
        .loopback()
        .add_service(
            ServiceBuilder::new("_oscjson._tcp.local.", &opts.app_name, http.port())
                .unwrap()
                .add_ip_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
                .add_txt("txtvers=1")
                .build()
                .unwrap(),
        )
        .build(IpVersion::V4)
        .unwrap()
        .run_in_background();

    // let service_type = zeroconf::ServiceType::new("osc", "udp").unwrap();
    // let mut browser = zeroconf::MdnsBrowser::new(service_type);
    // browser.set_service_callback(Box::new(handle_mdns_event));
    // let browser_event_loop = browser.browse_services().unwrap();
    // let shutdown = Arc::new(AtomicBool::new(true));
    // let shutdown1 = shutdown.clone();
    // let done = Arc::new(Notify::new());
    // let done1 = done.clone();
    //
    // thread::spawn(move || {
    //     while shutdown1.load(Ordering::Acquire) {
    //         browser_event_loop.poll(Duration::from_millis(100)).unwrap();
    //     }
    //     done1.notify_waiters();
    // });

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

    // shutdown.store(false, Ordering::Release);
    // done.notified().await;

    // zeroconf_tcp.stop().await;
    // zeroconf_udp.stop().await;
    // srv.shutdown().await;
    broadcaster.shutdown().unwrap();
    http.stop().await;
}

fn handle_packet(packet: OscPacket) {
    info!("Received OSC Packet: {:?}", packet);
}

// fn handle_mdns_event(
//     result: zeroconf::Result<zeroconf::BrowserEvent>,
//     _context: Option<Arc<dyn Any + Send + Sync>>,
// ) {
//     match result {
//         Ok(event) => {
//             info!("mDNS browser event: {:?}", event);
//         }
//         Err(err) => {
//             warn!("mDNS error event: {:?}", err);
//         }
//     }
// }
