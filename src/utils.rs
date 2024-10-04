use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use rand::Rng;
use serde_json::Value;
use tokio::{net::TcpSocket, time::timeout};

#[allow(dead_code)]
pub struct StatusWrap {
    pub ip: String,
    pub port: String,
    pub license: i64,
    pub version: String,
    pub description: String,
    pub online: i64,
    pub max_online: i64,
}

impl StatusWrap {
    pub fn from_value(value: &Value) -> Self {
        let ip = value["ip"].as_str().unwrap_or("err").to_string();
        let port = value["port"].as_str().unwrap_or("err").to_string();
        let license = value["license"].as_i64().unwrap_or(-1);
        let version = value["status"]["version"]["name"]
            .as_str()
            .unwrap_or("err")
            .to_string();
        let description = {
            let mut result = String::new();

            if let Some(extra) = value["status"]["description"]["extra"].as_array() {
                for element in extra {
                    if let Some(str) = element.as_str() {
                        result += str;
                    } else if let Some(str) = element["text"].as_str() {
                        result += str;
                    }
                }
            } else if let Some(text) = value["status"]["description"].as_str() {
                result += text;
            } else if let Some(text) = value["status"]["description"]["text"].as_str() {
                result += text;
            }

            result
        };
        let online = value["status"]["players"]["online"].as_i64().unwrap_or(-1);
        let max_online = value["status"]["players"]["max"].as_i64().unwrap_or(-1);

        StatusWrap {
            ip,
            port,
            license,
            version,
            description,
            online,
            max_online,
        }
    }
}

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
