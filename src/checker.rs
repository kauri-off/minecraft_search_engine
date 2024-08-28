use std::{
    io::{Error, Result},
    net::SocketAddr,
};

use serde_json::{json, Value};
use tokio::net::TcpSocket;

use crate::{
    packet::{packet::Packet, packet_builder::PacketBuilder},
    packets::packets::{Handshake, LoginStart, PacketActions, SetCompression, Status},
    types::var_int::VarInt,
};

pub async fn get_full_info(addr: SocketAddr) -> Result<Value> {
    let motd = get_status(addr).await?;
    let licensed = license(addr, motd["version"]["protocol"].as_i64().unwrap_or(765)).await;

    let license = match licensed {
        Ok(t) => {
            if t {
                1
            } else {
                0
            }
        }
        Err(_) => -1,
    };

    let mut info = json!({});
    info["ip"] = json!(addr.ip().to_string());
    info["port"] = json!(addr.port().to_string());
    info["license"] = json!(license);
    info["status"] = motd;

    Ok(info)
}

pub async fn get_status(addr: SocketAddr) -> Result<Value> {
    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;

    let handshake = Handshake {
        packet_id: VarInt(0x00),
        protocol_version: VarInt(765),
        server_address: addr.ip().to_string(),
        server_port: addr.port(),
        next_state: VarInt(0x01),
    };
    handshake.serialize().write(&mut stream).await?;

    let status_req = PacketBuilder::new(VarInt(0x00)).build();
    status_req.write(&mut stream).await?;

    let response = Packet::read_uncompressed(&mut stream).await?;
    let status = Status::deserialize(&response).await?;

    Ok(serde_json::from_str(&status.status)?)
}

pub async fn license(addr: SocketAddr, protocol: i64) -> Result<bool> {
    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;

    let handshake = Handshake {
        packet_id: VarInt(0x00),
        protocol_version: VarInt(protocol as i32),
        server_address: addr.ip().to_string(),
        server_port: addr.port(),
        next_state: VarInt(0x02),
    };
    handshake.serialize().write(&mut stream).await?;

    let login_start = LoginStart {
        packet_id: VarInt(0x00),
        name: "NotABot".to_string(),
        uuid: 0,
    }
    .serialize();

    login_start.write(&mut stream).await?;

    let packet = Packet::read_uncompressed(&mut stream).await?;

    if packet.packet_id.0 == 0x01 {
        return Ok(true);
    } else if packet.packet_id.0 == 0x02 {
        return Ok(false);
    } else if packet.packet_id.0 == 0x03 {
        let compression = SetCompression::deserialize(&packet).await?;

        let login_success = Packet::read(&mut stream, Some(compression.threshold.0)).await?;
        if login_success.packet_id().await?.0 == 0x02 {
            return Ok(false);
        } else {
            return Ok(true);
        }
    } else if packet.packet_id.0 == 0x00 {
        return Err(Error::new(std::io::ErrorKind::Other, "Disconnected"));
    } else {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            format!("Packet ID Error: {}", packet.packet_id.0),
        ));
    }
}
