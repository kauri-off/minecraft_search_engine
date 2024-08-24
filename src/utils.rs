use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use rand::Rng;
use tokio::{net::TcpSocket, time::timeout};

pub fn get_random_ip() -> IpAddr {
    let mut rng = rand::thread_rng();

    let ip = format!(
        "{}.{}.{}.{}",
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        rng.gen_range(0..256)
    );

    ip.parse().unwrap()
}

pub async fn check_port_open(ip: IpAddr, port: u16) -> bool {
    let socket = TcpSocket::new_v4().unwrap();

    if let Ok(stream) = timeout(
        Duration::from_secs(3),
        socket.connect(SocketAddr::new(ip, port)),
    )
    .await
    {
        if let Ok(_) = stream {
            true
        } else {
            false
        }
    } else {
        false
    }
}
