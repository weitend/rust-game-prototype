use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

mod client;
mod conditions;
mod config;
mod intent;
mod io;
mod lifecycle;
mod server;
mod setup;
mod spawn;
mod state;

use client::{client_apply_latest_snapshot, client_receive_packets, client_send_packets};
use conditions::{is_client_like_mode, is_server_like_mode};
use config::NetConfig;
use lifecycle::log_lifecycle_messages;
use server::{
    server_log_controlled_players, server_prune_stale_sessions, server_receive_packets,
    server_respawn_missing_players, server_send_snapshots,
};
use setup::{setup_client_network, setup_server_network};

pub use state::NetLifecycleMessage;

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetConfig::from_env())
            .add_message::<NetLifecycleMessage>()
            .add_systems(Startup, setup_server_network.run_if(is_server_like_mode))
            .add_systems(Startup, setup_client_network.run_if(is_client_like_mode))
            .add_systems(
                PreUpdate,
                (
                    server_receive_packets.run_if(is_server_like_mode),
                    client_receive_packets.run_if(is_client_like_mode),
                ),
            )
            .add_systems(
                Update,
                (
                    server_prune_stale_sessions.run_if(is_server_like_mode),
                    server_respawn_missing_players
                        .run_if(is_server_like_mode)
                        .after(server_prune_stale_sessions),
                    server_send_snapshots
                        .run_if(is_server_like_mode)
                        .after(server_respawn_missing_players)
                        .after(server_prune_stale_sessions),
                    server_log_controlled_players
                        .run_if(is_server_like_mode)
                        .run_if(on_timer(Duration::from_secs(1))),
                    client_send_packets.run_if(is_client_like_mode),
                    log_lifecycle_messages,
                ),
            )
            .add_systems(
                Update,
                client_apply_latest_snapshot
                    .run_if(is_client_like_mode),
            );
    }
}
