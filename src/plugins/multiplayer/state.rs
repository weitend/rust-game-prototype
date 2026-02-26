use std::{
    collections::{HashMap, HashSet, VecDeque},
    net::{SocketAddr, UdpSocket},
};

use bevy::prelude::*;

use crate::{
    components::intent::PlayerIntent,
    network::protocol::{NetEntityId, Snapshot},
};

#[derive(Resource)]
pub(super) struct ServerNetState {
    pub(super) socket: UdpSocket,
    pub(super) sessions: HashMap<SocketAddr, ServerSession>,
    pub(super) next_session_id: u64,
    pub(super) tick_counter: u32,
    pub(super) snapshot_tick: u32,
    pub(super) next_impact_seq: u32,
    pub(super) snapshot_timer: Timer,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ServerSession {
    pub(super) id: u64,
    pub(super) last_seen_secs: f64,
    pub(super) player_entity: Option<Entity>,
    pub(super) last_input_seq: Option<u32>,
    pub(super) was_present_last_snapshot: bool,
    pub(super) respawn_deadline_secs: Option<f64>,
    pub(super) is_host_local: bool,
}

#[derive(Resource)]
pub(super) struct ClientNetState {
    pub(super) socket: UdpSocket,
    pub(super) server_addr: SocketAddr,
    pub(super) session_id: Option<u64>,
    pub(super) connected: bool,
    pub(super) nonce: u64,
    pub(super) next_ping_seq: u32,
    pub(super) next_input_seq: u32,
    pub(super) last_sent_intent: Option<PlayerIntent>,
    pub(super) hello_timer: Timer,
    pub(super) ping_timer: Timer,
    pub(super) input_timer: Timer,
}

#[derive(Resource, Debug, Clone, Copy)]
pub(super) struct HostLoopbackNonce(pub(super) u64);

#[derive(Resource, Default)]
pub(super) struct ClientSnapshotState {
    pub(super) latest: Option<Snapshot>,
    pub(super) last_applied_tick: Option<u32>,
    pub(super) by_net_id: HashMap<NetEntityId, SnapshotReplicaEntities>,
    pub(super) mesh: Option<Handle<Mesh>>,
    pub(super) material: Option<Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
pub(super) struct ClientImpactSyncState {
    pub(super) seen_seqs: HashSet<u32>,
    pub(super) seq_order: VecDeque<u32>,
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct NetworkControlledPlayer {
    pub(super) session_id: u64,
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SnapshotReplica;

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SnapshotReplicaTurret;

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SnapshotReplicaBarrel;

#[derive(Debug, Clone, Copy)]
pub(super) struct SnapshotReplicaEntities {
    pub(super) root: Entity,
    pub(super) turret: Option<Entity>,
    pub(super) barrel: Option<Entity>,
}

pub(super) const CLIENT_IMPACT_SEQ_WINDOW: usize = 4096;

#[derive(Message, Debug, Clone)]
pub enum NetLifecycleMessage {
    ServerSessionConnected {
        session_id: u64,
        addr: SocketAddr,
    },
    ServerSessionDisconnected {
        session_id: u64,
        addr: SocketAddr,
        reason: String,
    },
    ClientConnected {
        session_id: u64,
        server_addr: SocketAddr,
    },
    ClientDisconnected {
        reason: String,
    },
}
