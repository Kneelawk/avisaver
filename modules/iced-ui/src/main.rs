use std::thread;
use std::time::Duration;

fn main() {
    let _server = avisaver_osc::zeroconf::ZeroconfServer::start(25569).unwrap();

    thread::sleep(Duration::from_secs(10));
}
