use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};

use bevy::prelude::*;

use crate::resources::run_mode::{AppRunMode, RunMode};

use super::{
    config::NetConfig,
    io::now_millis,
    state::{
        ClientImpactSyncState, ClientNetState, ClientNetworkTelemetry, ClientSnapshotState,
        HostLoopbackNonce, ServerNetState,
    },
};

pub(super) fn setup_server_network(mut commands: Commands, config: Res<NetConfig>) {
    match UdpSocket::bind(&config.server_bind_addr) {
        Ok(socket) => {
            if socket.set_nonblocking(true).is_err() {
                eprintln!(
                    "[net-server] failed to set nonblocking on {}",
                    config.server_bind_addr
                );
                return;
            }
            eprintln!("[net-server] listening on {}", config.server_bind_addr);
            commands.insert_resource(ServerNetState {
                socket,
                sessions: HashMap::new(),
                next_session_id: 1,
                tick_counter: 0,
                snapshot_tick: 0,
                next_impact_seq: 1,
                snapshot_timer: Timer::from_seconds(
                    config.snapshot_interval_secs,
                    TimerMode::Repeating,
                ),
            });
        }
        Err(err) => {
            eprintln!(
                "[net-server] bind failed at {}: {}",
                config.server_bind_addr, err
            );
        }
    }
}

pub(super) fn setup_client_network(
    mut commands: Commands,
    config: Res<NetConfig>,
    run_mode: Res<AppRunMode>,
) {
    let server_addr: SocketAddr = match config.server_addr.parse() {
        Ok(addr) => addr,
        Err(err) => {
            eprintln!(
                "[net-client] invalid server addr '{}': {}",
                config.server_addr, err
            );
            return;
        }
    };

    let socket = match UdpSocket::bind(&config.client_bind_addr) {
        Ok(socket) => socket,
        Err(err) => {
            eprintln!(
                "[net-client] bind failed at {}: {}",
                config.client_bind_addr, err
            );
            return;
        }
    };

    if socket.set_nonblocking(true).is_err() {
        eprintln!(
            "[net-client] failed to set nonblocking on {}",
            config.client_bind_addr
        );
        return;
    }

    let nonce = now_millis() as u64 ^ (std::process::id() as u64);
    let hello_timer = Timer::from_seconds(0.25, TimerMode::Repeating);
    let ping_timer = Timer::from_seconds(config.ping_interval_secs, TimerMode::Repeating);
    let input_timer = Timer::from_seconds(config.input_send_interval_secs, TimerMode::Repeating);

    eprintln!(
        "[net-client] bound at {}, server={}",
        config.client_bind_addr, config.server_addr
    );

    commands.insert_resource(ClientNetState {
        socket,
        server_addr,
        session_id: None,
        connected: false,
        nonce,
        next_ping_seq: 1,
        next_input_seq: 1,
        last_sent_intent: None,
        hello_timer,
        ping_timer,
        input_timer,
        pending_pings_secs: HashMap::new(),
        pings_sent: 0,
        pongs_received: 0,
    });
    if matches!(run_mode.0, RunMode::Host) {
        commands.insert_resource(HostLoopbackNonce(nonce));
    }
    commands.insert_resource(ClientSnapshotState::default());
    commands.insert_resource(ClientImpactSyncState::default());
    commands.insert_resource(ClientNetworkTelemetry::default());
}
