use std::{
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::network::protocol::{ClientPacket, ServerPacket};

pub(super) fn send_client_packet(socket: &UdpSocket, server_addr: SocketAddr, packet: &ClientPacket) {
    if let Ok(bytes) = bincode::serialize(packet) {
        let _ = socket.send_to(&bytes, server_addr);
    }
}

pub(super) fn send_server_packet(socket: &UdpSocket, addr: SocketAddr, packet: &ServerPacket) {
    if let Ok(bytes) = bincode::serialize(packet) {
        let _ = socket.send_to(&bytes, addr);
    }
}

pub(super) fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
