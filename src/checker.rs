use std::{
    io::{Error, Result},
    net::SocketAddr,
};

use serde_json::{Number, Value};
use tokio::net::TcpSocket;

use crate::{
    packet::{packet::Packet, packet_builder::PacketBuilder},
    packets::packets::{Handshake, LoginStart, PacketActions, SetCompression, Status},
    types::var_int::VarInt,
};

pub struct ServerStatus {
    pub version: String,
    pub motd: String,
    pub online: String,
    pub max_online: String,
    pub protocol: i32,
}

pub async fn get_motd(addr: SocketAddr) -> Result<ServerStatus> {
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

    let status: Value = serde_json::from_str(&status.status)?;

    Ok(ServerStatus {
        version: status["version"]["name"].as_str().unwrap_or("").to_string(),
        motd: status["description"].as_str().unwrap_or("").to_string(),
        online: status["players"]["online"]
            .as_number()
            .unwrap_or(&Number::from(-1))
            .to_string(),
        max_online: status["players"]["max"]
            .as_number()
            .unwrap_or(&Number::from(-1))
            .to_string(),
        protocol: status["version"]["protocol"].as_i64().unwrap_or(765) as i32,
    })
}

pub async fn license(addr: SocketAddr, protocol: i32) -> Result<bool> {
    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;

    let handshake = Handshake {
        packet_id: VarInt(0x00),
        protocol_version: VarInt(protocol),
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

    if packet.packet_id.0 == 0x02 {
        return Ok(false);
    } else if packet.packet_id.0 == 0x03 {
        let compression = SetCompression::deserialize(&packet).await?;

        let login_success = Packet::read(&mut stream, Some(compression.threshold.0)).await?;
        if login_success.packet_id().await?.0 == 0x02 {
            return Ok(false);
        } else {
            return Ok(true);
        }
    } else {
        return Err(Error::new(std::io::ErrorKind::Other, "Packet ID Error"));
    }
}
