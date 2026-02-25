use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_rapier3d::prelude::{
    CharacterAutostep, CharacterLength, Collider, Damping, ExternalForce,
    KinematicCharacterController, LockedAxes, RigidBody, Velocity,
};

use crate::{
    components::{
        combat::{Health, Team},
        fire_control::FireControl,
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player, PlayerControllerState},
        tank::{
            TankBarrel, TankBarrelState, TankHull, TankMuzzle, TankParts, TankTurret,
            TankTurretState,
        },
        weapon::HitscanWeapon,
    },
    network::protocol::{
        ClientInput, ClientPacket, EntitySnapshot, NetEntityId, PROTOCOL_VERSION, ServerPacket,
        Snapshot,
    },
    resources::run_mode::{AppRunMode, RunMode},
    resources::{
        player_motion_settings::PlayerMotionSettings,
        player_physics_settings::{PlayerHullPhysicsMode, PlayerPhysicsSettings},
        player_spawn::PlayerTemplate,
    },
};

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetConfig::from_env())
            .add_message::<NetLifecycleMessage>()
            .add_systems(Startup, setup_server_network.run_if(is_server_like_mode))
            .add_systems(Startup, setup_client_network.run_if(is_client_like_mode))
            .add_systems(
                Update,
                (
                    server_receive_packets.run_if(is_server_like_mode),
                    server_prune_stale_sessions.run_if(is_server_like_mode),
                    server_send_snapshots.run_if(is_server_like_mode),
                    server_log_controlled_players
                        .run_if(is_server_like_mode)
                        .run_if(on_timer(Duration::from_secs(1))),
                    client_send_packets.run_if(is_client_like_mode),
                    client_receive_packets.run_if(is_client_like_mode),
                    log_lifecycle_messages,
                ),
            )
            .add_systems(
                Update,
                client_apply_latest_snapshot
                    .run_if(is_client_like_mode)
                    .after(client_receive_packets),
            );
    }
}

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

#[derive(Resource)]
struct ServerNetState {
    socket: UdpSocket,
    sessions: HashMap<SocketAddr, ServerSession>,
    next_session_id: u64,
    tick_counter: u32,
    snapshot_tick: u32,
    snapshot_timer: Timer,
}

#[derive(Debug, Clone, Copy)]
struct ServerSession {
    id: u64,
    last_seen_secs: f64,
    player_entity: Option<Entity>,
    last_input_seq: Option<u32>,
}

#[derive(Resource)]
struct ClientNetState {
    socket: UdpSocket,
    server_addr: SocketAddr,
    session_id: Option<u64>,
    connected: bool,
    nonce: u64,
    next_ping_seq: u32,
    next_input_seq: u32,
    last_sent_intent: Option<PlayerIntent>,
    hello_timer: Timer,
    ping_timer: Timer,
    input_timer: Timer,
}

#[derive(Resource, Debug, Clone, Copy)]
struct HostLoopbackNonce(u64);

#[derive(Resource, Default)]
struct ClientSnapshotState {
    latest: Option<Snapshot>,
    by_net_id: HashMap<NetEntityId, SnapshotReplicaEntities>,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
    local_missing_streak: u32,
}

#[derive(Component, Debug, Clone, Copy)]
struct NetworkControlledPlayer {
    session_id: u64,
}

#[derive(Component, Debug, Clone, Copy)]
struct SnapshotReplica;

#[derive(Component, Debug, Clone, Copy)]
struct SnapshotReplicaTurret;

#[derive(Component, Debug, Clone, Copy)]
struct SnapshotReplicaBarrel;

#[derive(Debug, Clone, Copy)]
struct SnapshotReplicaEntities {
    root: Entity,
    turret: Option<Entity>,
    barrel: Option<Entity>,
}

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

fn setup_server_network(mut commands: Commands, config: Res<NetConfig>) {
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

fn setup_client_network(mut commands: Commands, config: Res<NetConfig>, run_mode: Res<AppRunMode>) {
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
    });
    if matches!(run_mode.0, RunMode::Host) {
        commands.insert_resource(HostLoopbackNonce(nonce));
    }
    commands.insert_resource(ClientSnapshotState::default());
}

fn server_receive_packets(
    mut commands: Commands,
    state: Option<ResMut<ServerNetState>>,
    time: Res<Time>,
    config: Res<NetConfig>,
    run_mode: Res<AppRunMode>,
    host_loopback_nonce: Option<Res<HostLoopbackNonce>>,
    motion_settings: Res<PlayerMotionSettings>,
    physics_settings: Res<PlayerPhysicsSettings>,
    mut lifecycle: MessageWriter<NetLifecycleMessage>,
    mut player_intent_q: Query<&mut PlayerIntent, With<Player>>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    network_player_q: Query<(), With<NetworkControlledPlayer>>,
) {
    let Some(mut state) = state else {
        return;
    };

    let now_secs = time.elapsed_secs_f64();
    let mut buffer = [0u8; 4096];

    loop {
        let recv = state.socket.recv_from(&mut buffer);
        let (len, addr) = match recv {
            Ok(v) => v,
            Err(err) if err.kind() == ErrorKind::WouldBlock => break,
            Err(err) => {
                eprintln!("[net-server] recv error: {}", err);
                break;
            }
        };

        let packet: ClientPacket = match bincode::deserialize(&buffer[..len]) {
            Ok(packet) => packet,
            Err(err) => {
                eprintln!("[net-server] invalid packet from {}: {}", addr, err);
                continue;
            }
        };

        state.tick_counter = state.tick_counter.wrapping_add(1);

        match packet {
            ClientPacket::Hello {
                protocol_version,
                nonce,
            } => {
                if protocol_version != config.protocol_version {
                    send_server_packet(
                        &state.socket,
                        addr,
                        &ServerPacket::Disconnect {
                            reason: format!(
                                "protocol mismatch: server={} client={}",
                                config.protocol_version, protocol_version
                            ),
                        },
                    );
                    continue;
                }

                let (session_id, is_new) = if let Some(session) = state.sessions.get_mut(&addr) {
                    session.last_seen_secs = now_secs;
                    (session.id, false)
                } else {
                    let session_id = state.next_session_id;
                    state.next_session_id = state.next_session_id.saturating_add(1);
                    let bind_to_host_local = matches!(run_mode.0, RunMode::Host)
                        && host_loopback_nonce
                            .as_ref()
                            .is_some_and(|local_nonce| local_nonce.0 == nonce);
                    let preferred_host_local = if bind_to_host_local {
                        first_unbound_local_player_entity(&state, &local_player_q)
                    } else {
                        None
                    };
                    let player_entity = assign_player_entity_for_session(
                        &mut commands,
                        &run_mode,
                        preferred_host_local,
                        &motion_settings,
                        &physics_settings,
                        session_id,
                    );
                    state.sessions.insert(
                        addr,
                        ServerSession {
                            id: session_id,
                            last_seen_secs: now_secs,
                            player_entity: Some(player_entity),
                            last_input_seq: None,
                        },
                    );
                    (session_id, true)
                };

                if is_new {
                    lifecycle
                        .write(NetLifecycleMessage::ServerSessionConnected { session_id, addr });
                }

                send_server_packet(
                    &state.socket,
                    addr,
                    &ServerPacket::Welcome {
                        protocol_version: config.protocol_version,
                        session_id,
                    },
                );
            }
            ClientPacket::Ping { seq } => {
                if let Some(session) = state.sessions.get_mut(&addr) {
                    session.last_seen_secs = now_secs;
                    send_server_packet(
                        &state.socket,
                        addr,
                        &ServerPacket::Pong {
                            seq,
                            server_tick: state.tick_counter,
                        },
                    );
                }
            }
            ClientPacket::Disconnect { reason } => {
                if let Some(session) = state.sessions.remove(&addr) {
                    despawn_network_owned_player(
                        &mut commands,
                        &network_player_q,
                        session.player_entity,
                    );
                    lifecycle.write(NetLifecycleMessage::ServerSessionDisconnected {
                        session_id: session.id,
                        addr,
                        reason,
                    });
                }
            }
            ClientPacket::Input(input) => {
                if let Some(session) = state.sessions.get_mut(&addr) {
                    session.last_seen_secs = now_secs;

                    if session.last_input_seq == Some(input.seq) {
                        continue;
                    }
                    session.last_input_seq = Some(input.seq);

                    let player_entity = if let Some(entity) = session.player_entity {
                        entity
                    } else {
                        let entity = assign_player_entity_for_session(
                            &mut commands,
                            &run_mode,
                            None,
                            &motion_settings,
                            &physics_settings,
                            session.id,
                        );
                        session.player_entity = Some(entity);
                        entity
                    };

                    if let Ok(mut intent) = player_intent_q.get_mut(player_entity) {
                        *intent = player_intent_from_client_input(&input);
                        if input.seq % 20 == 0 || input.fire_just_pressed {
                            eprintln!(
                                "[net-server] input applied: session_id={} entity={:?} seq={} throttle={:.2} turn={:.2}",
                                session.id, player_entity, input.seq, input.throttle, input.turn
                            );
                        }
                    }
                }
            }
        }
    }
}

fn server_send_snapshots(
    state: Option<ResMut<ServerNetState>>,
    time: Res<Time>,
    player_snapshot_q: Query<(&Transform, Option<&Health>, Option<&TankParts>)>,
    turret_state_q: Query<&TankTurretState, With<TankTurret>>,
    barrel_state_q: Query<&TankBarrelState, With<TankBarrel>>,
) {
    let Some(mut state) = state else {
        return;
    };

    state.snapshot_timer.tick(time.delta());
    if !state.snapshot_timer.just_finished() {
        return;
    }

    let mut entities = Vec::new();
    for session in state.sessions.values() {
        let Some(player_entity) = session.player_entity else {
            continue;
        };
        let Ok((tf, health, tank_parts)) = player_snapshot_q.get(player_entity) else {
            continue;
        };
        let (turret_yaw, barrel_pitch) = tank_parts
            .and_then(|parts| {
                let turret = turret_state_q.get(parts.turret).ok().map(|s| s.yaw);
                let barrel = barrel_state_q.get(parts.barrel).ok().map(|s| s.pitch);
                if turret.is_none() && barrel.is_none() {
                    None
                } else {
                    Some((turret, barrel))
                }
            })
            .unwrap_or((None, None));
        entities.push(EntitySnapshot {
            id: NetEntityId(session.id),
            position: [tf.translation.x, tf.translation.y, tf.translation.z],
            rotation: [tf.rotation.x, tf.rotation.y, tf.rotation.z, tf.rotation.w],
            health: health.map(|h| [h.current, h.max]),
            turret_yaw,
            barrel_pitch,
        });
    }

    state.snapshot_tick = state.snapshot_tick.wrapping_add(1);
    let snapshot = Snapshot {
        tick: state.snapshot_tick,
        entities,
        events: Vec::new(),
    };
    let addrs: Vec<SocketAddr> = state.sessions.keys().copied().collect();
    for addr in addrs {
        send_server_packet(
            &state.socket,
            addr,
            &ServerPacket::Snapshot(snapshot.clone()),
        );
    }
}

fn server_prune_stale_sessions(
    mut commands: Commands,
    state: Option<ResMut<ServerNetState>>,
    time: Res<Time>,
    config: Res<NetConfig>,
    mut lifecycle: MessageWriter<NetLifecycleMessage>,
    network_player_q: Query<(), With<NetworkControlledPlayer>>,
) {
    let Some(mut state) = state else {
        return;
    };

    let now_secs = time.elapsed_secs_f64();
    let mut stale = Vec::new();
    for (addr, session) in &state.sessions {
        if now_secs - session.last_seen_secs > config.session_timeout_secs as f64 {
            stale.push((*addr, session.id, session.player_entity));
        }
    }

    for (addr, session_id, player_entity) in stale {
        state.sessions.remove(&addr);
        despawn_network_owned_player(&mut commands, &network_player_q, player_entity);
        lifecycle.write(NetLifecycleMessage::ServerSessionDisconnected {
            session_id,
            addr,
            reason: "timeout".to_string(),
        });
    }
}

fn client_send_packets(
    state: Option<ResMut<ClientNetState>>,
    time: Res<Time>,
    config: Res<NetConfig>,
    local_player_intent_q: Query<&PlayerIntent, (With<Player>, With<LocalPlayer>)>,
) {
    let Some(mut state) = state else {
        return;
    };

    state.hello_timer.tick(time.delta());
    state.ping_timer.tick(time.delta());
    state.input_timer.tick(time.delta());

    if !state.connected {
        if state.hello_timer.just_finished() {
            send_client_packet(
                &state.socket,
                state.server_addr,
                &ClientPacket::Hello {
                    protocol_version: config.protocol_version,
                    nonce: state.nonce,
                },
            );
        }
        return;
    }

    if state.ping_timer.just_finished() {
        let seq = state.next_ping_seq;
        state.next_ping_seq = state.next_ping_seq.wrapping_add(1);
        send_client_packet(
            &state.socket,
            state.server_addr,
            &ClientPacket::Ping { seq },
        );
    }

    let Some(intent) = read_single_local_intent(&local_player_intent_q) else {
        return;
    };
    let input_tick_due = state.input_timer.just_finished();
    let input_changed = state
        .last_sent_intent
        .is_none_or(|prev| intent_changed_significantly(prev, intent));
    let should_send_input = input_tick_due || intent.fire_just_pressed || input_changed;

    if should_send_input {
        let seq = state.next_input_seq;
        state.next_input_seq = state.next_input_seq.wrapping_add(1);
        send_client_packet(
            &state.socket,
            state.server_addr,
            &ClientPacket::Input(ClientInput {
                seq,
                throttle: intent.throttle,
                turn: intent.turn,
                turret_yaw_delta: intent.turret_yaw_delta,
                barrel_pitch_delta: intent.barrel_pitch_delta,
                fire_pressed: intent.fire_pressed,
                fire_just_pressed: intent.fire_just_pressed,
                artillery_active: intent.artillery_active,
            }),
        );
        state.last_sent_intent = Some(intent);
        if !input_tick_due {
            state.input_timer.reset();
        }
    }
}

fn client_receive_packets(
    state: Option<ResMut<ClientNetState>>,
    snapshot_state: Option<ResMut<ClientSnapshotState>>,
    mut lifecycle: MessageWriter<NetLifecycleMessage>,
) {
    let Some(mut state) = state else {
        return;
    };
    let mut snapshot_state = snapshot_state;

    let mut buffer = [0u8; 4096];
    loop {
        let recv = state.socket.recv_from(&mut buffer);
        let (len, addr) = match recv {
            Ok(v) => v,
            Err(err) if err.kind() == ErrorKind::WouldBlock => break,
            Err(err) => {
                eprintln!("[net-client] recv error: {}", err);
                break;
            }
        };

        if addr != state.server_addr {
            continue;
        }

        let packet: ServerPacket = match bincode::deserialize(&buffer[..len]) {
            Ok(packet) => packet,
            Err(err) => {
                eprintln!("[net-client] invalid server packet: {}", err);
                continue;
            }
        };

        match packet {
            ServerPacket::Welcome {
                protocol_version,
                session_id,
            } => {
                if protocol_version != PROTOCOL_VERSION {
                    lifecycle.write(NetLifecycleMessage::ClientDisconnected {
                        reason: format!(
                            "protocol mismatch: server={} client={}",
                            protocol_version, PROTOCOL_VERSION
                        ),
                    });
                    continue;
                }
                if !state.connected || state.session_id != Some(session_id) {
                    state.connected = true;
                    state.session_id = Some(session_id);
                    lifecycle.write(NetLifecycleMessage::ClientConnected {
                        session_id,
                        server_addr: state.server_addr,
                    });
                }
            }
            ServerPacket::Pong {
                seq: _,
                server_tick: _,
            } => {}
            ServerPacket::Disconnect { reason } => {
                if state.connected {
                    lifecycle.write(NetLifecycleMessage::ClientDisconnected { reason });
                }
                state.connected = false;
                state.session_id = None;
                state.next_input_seq = 1;
                state.last_sent_intent = None;
                if let Some(snapshot_state) = snapshot_state.as_deref_mut() {
                    snapshot_state.latest = Some(Snapshot {
                        tick: 0,
                        entities: Vec::new(),
                        events: Vec::new(),
                    });
                    snapshot_state.local_missing_streak = 0;
                }
            }
            ServerPacket::Snapshot(snapshot) => {
                if let Some(snapshot_state) = snapshot_state.as_deref_mut() {
                    snapshot_state.latest = Some(snapshot);
                }
            }
            ServerPacket::Event(_) => {}
        }
    }
}

fn log_lifecycle_messages(mut messages: MessageReader<NetLifecycleMessage>) {
    for message in messages.read() {
        match message {
            NetLifecycleMessage::ServerSessionConnected { session_id, addr } => {
                eprintln!(
                    "[net-server] client connected: session_id={} addr={}",
                    session_id, addr
                );
            }
            NetLifecycleMessage::ServerSessionDisconnected {
                session_id,
                addr,
                reason,
            } => {
                eprintln!(
                    "[net-server] client disconnected: session_id={} addr={} reason={}",
                    session_id, addr, reason
                );
            }
            NetLifecycleMessage::ClientConnected {
                session_id,
                server_addr,
            } => {
                eprintln!(
                    "[net-client] connected: session_id={} server={}",
                    session_id, server_addr
                );
            }
            NetLifecycleMessage::ClientDisconnected { reason } => {
                eprintln!("[net-client] disconnected: reason={}", reason);
            }
        }
    }
}

fn client_apply_latest_snapshot(
    mut commands: Commands,
    run_mode: Res<AppRunMode>,
    snapshot_state: Option<ResMut<ClientSnapshotState>>,
    client_state: Option<Res<ClientNetState>>,
    mut replica_tf_q: Query<
        &mut Transform,
        (
            With<SnapshotReplica>,
            Without<LocalPlayer>,
            Without<SnapshotReplicaTurret>,
            Without<SnapshotReplicaBarrel>,
        ),
    >,
    mut replica_turret_tf_q: Query<
        &mut Transform,
        (
            With<SnapshotReplicaTurret>,
            Without<SnapshotReplicaBarrel>,
            Without<Player>,
            Without<SnapshotReplica>,
        ),
    >,
    mut replica_barrel_tf_q: Query<
        &mut Transform,
        (
            With<SnapshotReplicaBarrel>,
            Without<SnapshotReplicaTurret>,
            Without<Player>,
            Without<SnapshotReplica>,
        ),
    >,
    mut local_player_q: Query<
        (Entity, &mut Transform, Option<&mut Health>),
        (With<Player>, With<LocalPlayer>, Without<SnapshotReplica>),
    >,
    player_template: Option<Res<PlayerTemplate>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut snapshot_state) = snapshot_state else {
        return;
    };
    let Some(snapshot) = snapshot_state.latest.take() else {
        return;
    };

    if snapshot_state.mesh.is_none() {
        snapshot_state.mesh = Some(meshes.add(Cuboid::new(1.2, 0.7, 1.8)));
    }
    if snapshot_state.material.is_none() {
        snapshot_state.material = Some(materials.add(Color::srgb_u8(246, 168, 20)));
    }

    let local_session = client_state.as_ref().and_then(|state| state.session_id);
    let Some(mesh) = snapshot_state.mesh.clone() else {
        return;
    };
    let Some(material) = snapshot_state.material.clone() else {
        return;
    };

    let mut seen_ids = HashSet::new();
    let mut seen_local = false;
    let tick = snapshot.tick;
    for entity_snapshot in snapshot.entities {
        if local_session == Some(entity_snapshot.id.0) {
            seen_local = true;
            if !matches!(run_mode.0, RunMode::Host)
                && let Ok((_, mut local_tf, maybe_health)) = local_player_q.single_mut()
            {
                let server_translation = Vec3::new(
                    entity_snapshot.position[0],
                    entity_snapshot.position[1],
                    entity_snapshot.position[2],
                );
                let server_rotation = normalized_rotation(entity_snapshot.rotation);
                reconcile_local_transform(&mut local_tf, server_translation, server_rotation);
                if let (Some([current, max]), Some(mut health)) =
                    (entity_snapshot.health, maybe_health)
                {
                    health.max = max.max(0.0);
                    health.current = current.clamp(0.0, health.max);
                }
            }
            continue;
        }

        seen_ids.insert(entity_snapshot.id);
        let next_translation = Vec3::new(
            entity_snapshot.position[0],
            entity_snapshot.position[1],
            entity_snapshot.position[2],
        );
        let next_rotation = normalized_rotation(entity_snapshot.rotation);

        if let Some(replica) = snapshot_state.by_net_id.get(&entity_snapshot.id).copied() {
            if let Ok(mut tf) = replica_tf_q.get_mut(replica.root) {
                tf.translation = next_translation;
                tf.rotation = next_rotation;
            }
            if let (Some(turret_entity), Some(turret_yaw)) =
                (replica.turret, entity_snapshot.turret_yaw)
                && let Ok(mut turret_tf) = replica_turret_tf_q.get_mut(turret_entity)
            {
                turret_tf.rotation = Quat::from_rotation_y(turret_yaw);
            }
            if let (Some(barrel_entity), Some(barrel_pitch)) =
                (replica.barrel, entity_snapshot.barrel_pitch)
                && let Ok(mut barrel_tf) = replica_barrel_tf_q.get_mut(barrel_entity)
            {
                barrel_tf.rotation = Quat::from_rotation_x(barrel_pitch);
            }
        } else {
            let replica = spawn_snapshot_replica(
                &mut commands,
                entity_snapshot.id,
                next_translation,
                next_rotation,
                entity_snapshot.turret_yaw,
                entity_snapshot.barrel_pitch,
                player_template.as_deref(),
                mesh.clone(),
                material.clone(),
            );
            snapshot_state.by_net_id.insert(entity_snapshot.id, replica);
        }
    }

    let stale_ids: Vec<NetEntityId> = snapshot_state
        .by_net_id
        .keys()
        .copied()
        .filter(|id| !seen_ids.contains(id))
        .collect();
    for stale_id in stale_ids {
        if let Some(replica) = snapshot_state.by_net_id.remove(&stale_id) {
            commands.entity(replica.root).despawn_children().despawn();
        }
    }

    if !matches!(run_mode.0, RunMode::Host) && local_session.is_some() {
        if seen_local {
            snapshot_state.local_missing_streak = 0;
        } else {
            snapshot_state.local_missing_streak =
                snapshot_state.local_missing_streak.saturating_add(1);
            const LOCAL_DESPAWN_MISSING_SNAPSHOTS: u32 = 15;
            if snapshot_state.local_missing_streak >= LOCAL_DESPAWN_MISSING_SNAPSHOTS
                && let Ok((local_entity, _, _)) = local_player_q.single_mut()
            {
                commands.entity(local_entity).despawn_children().despawn();
                snapshot_state.local_missing_streak = 0;
            }
        }
    } else {
        snapshot_state.local_missing_streak = 0;
    }

    if tick % 30 == 0 {
        eprintln!(
            "[net-client] snapshot applied: tick={} replicas={}",
            tick,
            snapshot_state.by_net_id.len()
        );
    }
}

fn spawn_snapshot_replica(
    commands: &mut Commands,
    net_id: NetEntityId,
    translation: Vec3,
    rotation: Quat,
    turret_yaw: Option<f32>,
    barrel_pitch: Option<f32>,
    player_template: Option<&PlayerTemplate>,
    fallback_mesh: Handle<Mesh>,
    fallback_material: Handle<StandardMaterial>,
) -> SnapshotReplicaEntities {
    const TURRET_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.46, 0.0);
    const BARREL_PIVOT_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.09, -0.44);
    const BARREL_VISUAL_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.0, -0.63);

    if let Some(template) = player_template {
        let root = commands
            .spawn((
                Name::new(format!("NetReplica#{}", net_id.0)),
                Mesh3d(template.mesh.clone()),
                MeshMaterial3d(template.material.clone()),
                Transform::from_translation(translation).with_rotation(rotation),
                SnapshotReplica,
            ))
            .id();

        let turret = commands
            .spawn((
                Name::new(format!("NetReplica#{}::Turret", net_id.0)),
                Mesh3d(template.turret_mesh.clone()),
                MeshMaterial3d(template.turret_material.clone()),
                Transform::from_translation(TURRET_LOCAL_OFFSET)
                    .with_rotation(Quat::from_rotation_y(turret_yaw.unwrap_or(0.0))),
                SnapshotReplicaTurret,
            ))
            .id();
        let barrel_pivot = commands
            .spawn((
                Name::new(format!("NetReplica#{}::BarrelPivot", net_id.0)),
                Transform::from_translation(BARREL_PIVOT_LOCAL_OFFSET)
                    .with_rotation(Quat::from_rotation_x(barrel_pitch.unwrap_or(0.0))),
                Visibility::default(),
                SnapshotReplicaBarrel,
            ))
            .id();
        let barrel_visual = commands
            .spawn((
                Name::new(format!("NetReplica#{}::Barrel", net_id.0)),
                Mesh3d(template.barrel_mesh.clone()),
                MeshMaterial3d(template.barrel_material.clone()),
                Transform::from_translation(BARREL_VISUAL_LOCAL_OFFSET),
            ))
            .id();

        commands.entity(barrel_pivot).add_child(barrel_visual);
        commands.entity(turret).add_child(barrel_pivot);
        commands.entity(root).add_child(turret);
        return SnapshotReplicaEntities {
            root,
            turret: Some(turret),
            barrel: Some(barrel_pivot),
        };
    }

    let root = commands
        .spawn((
            Name::new(format!("NetReplica#{}", net_id.0)),
            Mesh3d(fallback_mesh),
            MeshMaterial3d(fallback_material),
            Transform::from_translation(translation).with_rotation(rotation),
            SnapshotReplica,
        ))
        .id();
    SnapshotReplicaEntities {
        root,
        turret: None,
        barrel: None,
    }
}

fn normalized_rotation(rotation: [f32; 4]) -> Quat {
    let mut quat = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);
    if quat.length_squared() > f32::EPSILON {
        quat = quat.normalize();
    } else {
        quat = Quat::IDENTITY;
    }
    quat
}

fn reconcile_local_transform(
    local_tf: &mut Transform,
    server_translation: Vec3,
    server_rotation: Quat,
) {
    let delta = server_translation - local_tf.translation;
    let distance = delta.length();
    const HARD_SNAP_DISTANCE: f32 = 2.5;
    const POS_BLEND: f32 = 0.35;
    const ROT_BLEND: f32 = 0.5;

    if distance > HARD_SNAP_DISTANCE {
        local_tf.translation = server_translation;
    } else {
        local_tf.translation += delta * POS_BLEND;
    }
    local_tf.rotation = local_tf.rotation.slerp(server_rotation, ROT_BLEND);
}

fn server_log_controlled_players(
    players: Query<(&Transform, &NetworkControlledPlayer), With<Player>>,
) {
    for (tf, network_owner) in &players {
        eprintln!(
            "[net-server] sim: session_id={} position=({:.2}, {:.2}, {:.2})",
            network_owner.session_id, tf.translation.x, tf.translation.y, tf.translation.z
        );
    }
}

fn assign_player_entity_for_session(
    commands: &mut Commands,
    run_mode: &AppRunMode,
    preferred_host_local: Option<Entity>,
    motion_settings: &PlayerMotionSettings,
    physics_settings: &PlayerPhysicsSettings,
    session_id: u64,
) -> Entity {
    if matches!(run_mode.0, RunMode::Host) {
        if let Some(local_player_entity) = preferred_host_local {
            eprintln!(
                "[net-server] host bind: session_id={} entity={:?}",
                session_id, local_player_entity
            );
            return local_player_entity;
        }
    }
    spawn_network_controlled_player(commands, motion_settings, physics_settings, session_id)
}

fn first_unbound_local_player_entity(
    state: &ServerNetState,
    local_player_q: &Query<Entity, (With<Player>, With<LocalPlayer>)>,
) -> Option<Entity> {
    local_player_q.iter().find(|candidate| {
        !state
            .sessions
            .values()
            .any(|s| s.player_entity == Some(*candidate))
    })
}

fn spawn_network_controlled_player(
    commands: &mut Commands,
    motion_settings: &PlayerMotionSettings,
    physics_settings: &PlayerPhysicsSettings,
    session_id: u64,
) -> Entity {
    const TURRET_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.46, 0.0);
    const BARREL_PIVOT_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.09, -0.44);
    const MUZZLE_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.0, -1.26);

    let spawn_x = ((session_id.saturating_sub(1)) as f32) * 3.5;
    let mut entity = commands.spawn((
        Name::new(format!("NetPlayer#{session_id}")),
        Transform::from_translation(Vec3::new(spawn_x, 0.9, 6.0)),
        Player,
        TankHull,
        Team::Player,
        Health::new(100.0),
        PlayerControllerState::default(),
        PlayerIntent::default(),
        FireControl {
            cooldown: Timer::from_seconds(1.0 / 5.0, TimerMode::Repeating),
        },
        HitscanWeapon {
            damage: 25.0,
            range: 45.0,
        },
        NetworkControlledPlayer { session_id },
        Collider::cuboid(0.80, 0.37, 1.10),
    ));

    match physics_settings.mode {
        PlayerHullPhysicsMode::KinematicController => {
            entity.insert(default_tank_controller(motion_settings));
        }
        PlayerHullPhysicsMode::DynamicForces => {
            entity.insert((
                RigidBody::Dynamic,
                Velocity::zero(),
                ExternalForce::default(),
                Damping {
                    linear_damping: physics_settings.dynamic_linear_damping,
                    angular_damping: physics_settings.dynamic_angular_damping,
                },
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            ));
        }
    }

    let id = entity.id();
    let turret = commands
        .spawn((
            Name::new(format!("NetPlayer#{session_id}::Turret")),
            Transform::from_translation(TURRET_LOCAL_OFFSET),
            OwnedBy { entity: id },
            TankTurret,
            TankTurretState::default(),
        ))
        .id();
    let barrel = commands
        .spawn((
            Name::new(format!("NetPlayer#{session_id}::BarrelPivot")),
            Transform::from_translation(BARREL_PIVOT_LOCAL_OFFSET),
            Visibility::default(),
            OwnedBy { entity: id },
            TankBarrel,
            TankBarrelState::default(),
        ))
        .id();
    let muzzle = commands
        .spawn((
            Name::new(format!("NetPlayer#{session_id}::Muzzle")),
            Transform::from_translation(MUZZLE_LOCAL_OFFSET),
            Visibility::default(),
            OwnedBy { entity: id },
            TankMuzzle,
        ))
        .id();
    commands.entity(barrel).add_child(muzzle);
    commands.entity(turret).add_child(barrel);
    commands.entity(id).add_child(turret);
    commands.entity(id).insert(TankParts {
        turret,
        barrel,
        muzzle,
    });

    eprintln!(
        "[net-server] spawned network player: session_id={} entity={:?}",
        session_id, id
    );
    id
}

fn default_tank_controller(settings: &PlayerMotionSettings) -> KinematicCharacterController {
    KinematicCharacterController {
        offset: CharacterLength::Absolute(settings.controller_offset),
        slide: true,
        apply_impulse_to_dynamic_bodies: false,
        filter_flags: bevy_rapier3d::prelude::QueryFilterFlags::EXCLUDE_DYNAMIC
            | bevy_rapier3d::prelude::QueryFilterFlags::EXCLUDE_SENSORS,
        autostep: Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(settings.autostep_height),
            min_width: CharacterLength::Absolute(settings.autostep_min_width),
            include_dynamic_bodies: false,
        }),
        snap_to_ground: Some(CharacterLength::Absolute(settings.snap_to_ground)),
        ..default()
    }
}

fn player_intent_from_client_input(input: &ClientInput) -> PlayerIntent {
    PlayerIntent {
        throttle: input.throttle.clamp(-1.0, 1.0),
        turn: input.turn.clamp(-1.0, 1.0),
        turret_yaw_delta: input.turret_yaw_delta,
        barrel_pitch_delta: input.barrel_pitch_delta,
        fire_pressed: input.fire_pressed,
        fire_just_pressed: input.fire_just_pressed,
        artillery_active: input.artillery_active,
    }
}

fn read_single_local_intent(
    local_player_intent_q: &Query<&PlayerIntent, (With<Player>, With<LocalPlayer>)>,
) -> Option<PlayerIntent> {
    let mut intents = local_player_intent_q.iter();
    let first = intents.next().copied()?;
    if intents.next().is_some() {
        return None;
    }
    Some(first)
}

fn intent_changed_significantly(prev: PlayerIntent, next: PlayerIntent) -> bool {
    const EPS: f32 = 0.001;
    (prev.throttle - next.throttle).abs() > EPS
        || (prev.turn - next.turn).abs() > EPS
        || (prev.turret_yaw_delta - next.turret_yaw_delta).abs() > EPS
        || (prev.barrel_pitch_delta - next.barrel_pitch_delta).abs() > EPS
        || prev.fire_pressed != next.fire_pressed
        || prev.fire_just_pressed != next.fire_just_pressed
        || prev.artillery_active != next.artillery_active
}

fn despawn_network_owned_player(
    commands: &mut Commands,
    network_player_q: &Query<(), With<NetworkControlledPlayer>>,
    player_entity: Option<Entity>,
) {
    let Some(player_entity) = player_entity else {
        return;
    };
    if network_player_q.get(player_entity).is_ok() {
        commands.entity(player_entity).despawn();
    }
}

fn send_client_packet(socket: &UdpSocket, server_addr: SocketAddr, packet: &ClientPacket) {
    if let Ok(bytes) = bincode::serialize(packet) {
        let _ = socket.send_to(&bytes, server_addr);
    }
}

fn send_server_packet(socket: &UdpSocket, addr: SocketAddr, packet: &ServerPacket) {
    if let Ok(bytes) = bincode::serialize(packet) {
        let _ = socket.send_to(&bytes, addr);
    }
}

fn is_server_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Server | RunMode::Host)
}

fn is_client_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Client | RunMode::Host)
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
