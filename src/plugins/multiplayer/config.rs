use bevy::prelude::Resource;

use crate::network::protocol::PROTOCOL_VERSION;

#[derive(Resource, Debug, Clone)]
pub struct NetConfig {
    pub server_bind_addr: String,
    pub client_bind_addr: String,
    pub server_addr: String,
    pub protocol_version: u16,
    pub ping_interval_secs: f32,
    pub input_send_interval_secs: f32,
    pub snapshot_interval_secs: f32,
    pub session_timeout_secs: f32,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            server_bind_addr: "0.0.0.0:47000".to_string(),
            client_bind_addr: "0.0.0.0:0".to_string(),
            server_addr: "127.0.0.1:47000".to_string(),
            protocol_version: PROTOCOL_VERSION,
            ping_interval_secs: 0.5,
            input_send_interval_secs: 1.0 / 60.0,
            snapshot_interval_secs: 1.0 / 60.0,
            session_timeout_secs: 5.0,
        }
    }
}

impl NetConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("RUST_GAME_SERVER_BIND_ADDR") {
            cfg.server_bind_addr = v;
        }
        if let Ok(v) = std::env::var("RUST_GAME_CLIENT_BIND_ADDR") {
            cfg.client_bind_addr = v;
        }
        if let Ok(v) = std::env::var("RUST_GAME_SERVER_ADDR") {
            cfg.server_addr = v;
        }
        if let Ok(v) = std::env::var("RUST_GAME_INPUT_SEND_INTERVAL_SECS") {
            if let Ok(parsed) = v.parse::<f32>() {
                cfg.input_send_interval_secs = parsed.max(0.01);
            }
        }
        if let Ok(v) = std::env::var("RUST_GAME_SNAPSHOT_INTERVAL_SECS") {
            if let Ok(parsed) = v.parse::<f32>() {
                cfg.snapshot_interval_secs = parsed.max(0.01);
            }
        }
        cfg
    }
}
