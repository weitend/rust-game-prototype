use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, Sensor};

use crate::{
    components::{
        combat::Health,
        obstacle::ObstacleNetId,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankParts, TankTurret, TankTurretState},
    },
    network::protocol::{
        ClientInput, ClientPacket, NetEntityId, PROTOCOL_VERSION, ServerEventDto, ServerPacket,
        Snapshot,
    },
    resources::{
        player_physics_settings::PlayerPhysicsSettings,
        player_spawn::PlayerTemplate,
        run_mode::{AppRunMode, RunMode},
    },
    systems::{impact::ImpactEvent, player_respawn::spawn_player_from_template},
};

use super::{
    config::NetConfig,
    intent::{intent_changed_significantly, read_single_local_intent},
    io::send_client_packet,
    state::{
        CLIENT_IMPACT_SEQ_WINDOW, ClientImpactSyncState, ClientNetState, ClientNetworkTelemetry,
        ClientSnapshotState, NetLifecycleMessage, SnapshotReplica, SnapshotReplicaBarrel,
        SnapshotReplicaEntities, SnapshotReplicaTurret,
    },
};

const PING_TIMEOUT_SECS: f64 = 3.0;

pub(super) fn client_send_packets(
    state: Option<ResMut<ClientNetState>>,
    telemetry: Option<ResMut<ClientNetworkTelemetry>>,
    time: Res<Time>,
    config: Res<NetConfig>,
    local_player_intent_q: Query<
        &crate::components::intent::PlayerIntent,
        (With<Player>, With<LocalPlayer>),
    >,
) {
    let Some(mut state) = state else {
        return;
    };
    let mut telemetry = telemetry;

    state.hello_timer.tick(time.delta());
    state.ping_timer.tick(time.delta());
    state.input_timer.tick(time.delta());

    if !state.connected {
        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.connected = false;
            telemetry.status = format!("net: connecting ({})", state.server_addr);
        }
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
        let now_secs = time.elapsed_secs_f64();
        state.pending_pings_secs.insert(seq, now_secs);
        state
            .pending_pings_secs
            .retain(|_, sent_at| now_secs - *sent_at <= PING_TIMEOUT_SECS);
        state.pings_sent = state.pings_sent.wrapping_add(1);
        send_client_packet(
            &state.socket,
            state.server_addr,
            &ClientPacket::Ping { seq },
        );
        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.packet_loss_pct =
                compute_packet_loss_pct(state.pings_sent, state.pongs_received);
        }
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

pub(super) fn client_receive_packets(
    state: Option<ResMut<ClientNetState>>,
    telemetry: Option<ResMut<ClientNetworkTelemetry>>,
    snapshot_state: Option<ResMut<ClientSnapshotState>>,
    impact_sync_state: Option<ResMut<ClientImpactSyncState>>,
    time: Res<Time>,
    mut lifecycle: MessageWriter<NetLifecycleMessage>,
) {
    let Some(mut state) = state else {
        return;
    };
    let mut telemetry = telemetry;
    let mut snapshot_state = snapshot_state;
    let mut impact_sync_state = impact_sync_state;

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
                    if let Some(telemetry) = telemetry.as_deref_mut() {
                        telemetry.connected = false;
                        telemetry.status = format!(
                            "net: protocol mismatch (srv={} cli={})",
                            protocol_version, PROTOCOL_VERSION
                        );
                    }
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
                    if let Some(telemetry) = telemetry.as_deref_mut() {
                        telemetry.connected = true;
                        telemetry.status = format!("net: connected (session {})", session_id);
                    }
                    lifecycle.write(NetLifecycleMessage::ClientConnected {
                        session_id,
                        server_addr: state.server_addr,
                    });
                }
            }
            ServerPacket::Pong { seq, server_tick } => {
                let now_secs = time.elapsed_secs_f64();
                let ping_ms = state
                    .pending_pings_secs
                    .remove(&seq)
                    .map(|sent_secs| ((now_secs - sent_secs).max(0.0) * 1000.0) as f32);
                state.pongs_received = state.pongs_received.wrapping_add(1);
                if let Some(telemetry) = telemetry.as_deref_mut() {
                    telemetry.connected = state.connected;
                    telemetry.ping_ms = ping_ms.or(telemetry.ping_ms);
                    telemetry.server_tick = Some(server_tick);
                    telemetry.packet_loss_pct =
                        compute_packet_loss_pct(state.pings_sent, state.pongs_received);
                    telemetry.status = "net: connected".to_owned();
                }
            }
            ServerPacket::Disconnect { reason } => {
                let reason_text = reason.clone();
                if state.connected {
                    lifecycle.write(NetLifecycleMessage::ClientDisconnected { reason });
                }
                state.connected = false;
                state.session_id = None;
                state.next_input_seq = 1;
                state.last_sent_intent = None;
                state.pending_pings_secs.clear();
                state.pings_sent = 0;
                state.pongs_received = 0;
                if let Some(snapshot_state) = snapshot_state.as_deref_mut() {
                    snapshot_state.latest = Some(Snapshot {
                        tick: 0,
                        entities: Vec::new(),
                        events: Vec::new(),
                    });
                    snapshot_state.last_applied_tick = None;
                }
                if let Some(impact_sync_state) = impact_sync_state.as_deref_mut() {
                    impact_sync_state.seen_seqs.clear();
                    impact_sync_state.seq_order.clear();
                }
                if let Some(telemetry) = telemetry.as_deref_mut() {
                    telemetry.connected = false;
                    telemetry.ping_ms = None;
                    telemetry.server_tick = None;
                    telemetry.packet_loss_pct = None;
                    telemetry.status = format!("net: disconnected ({})", reason_text);
                }
            }
            ServerPacket::Snapshot(snapshot) => {
                if let Some(snapshot_state) = snapshot_state.as_deref_mut() {
                    let is_newer_than_applied = snapshot_state
                        .last_applied_tick
                        .is_none_or(|last| is_snapshot_tick_newer(snapshot.tick, last));
                    if !is_newer_than_applied {
                        continue;
                    }

                    let should_replace_latest = snapshot_state
                        .latest
                        .as_ref()
                        .is_none_or(|latest| is_snapshot_tick_newer(snapshot.tick, latest.tick));
                    if should_replace_latest {
                        snapshot_state.latest = Some(snapshot);
                    }
                }
            }
            ServerPacket::Event(_) => {}
        }
    }
}

pub(super) fn client_apply_latest_snapshot(
    mut commands: Commands,
    mut impact_events: MessageWriter<ImpactEvent>,
    run_mode: Res<AppRunMode>,
    physics_settings: Res<PlayerPhysicsSettings>,
    snapshot_state: Option<ResMut<ClientSnapshotState>>,
    client_state: Option<Res<ClientNetState>>,
    impact_sync_state: Option<ResMut<ClientImpactSyncState>>,
    obstacle_q: Query<(Entity, &ObstacleNetId, &GlobalTransform)>,
    mut transform_sets: ParamSet<(
        Query<
            'static,
            'static,
            &mut Transform,
            (
                With<SnapshotReplica>,
                Without<LocalPlayer>,
                Without<SnapshotReplicaTurret>,
                Without<SnapshotReplicaBarrel>,
            ),
        >,
        Query<
            'static,
            'static,
            &mut Transform,
            (
                With<SnapshotReplicaTurret>,
                Without<SnapshotReplicaBarrel>,
                Without<Player>,
                Without<SnapshotReplica>,
            ),
        >,
        Query<
            'static,
            'static,
            &mut Transform,
            (
                With<SnapshotReplicaBarrel>,
                Without<SnapshotReplicaTurret>,
                Without<Player>,
                Without<SnapshotReplica>,
            ),
        >,
        Query<
            'static,
            'static,
            (
                &'static mut Transform,
                &'static mut TankTurretState,
                &'static OwnedBy,
            ),
            (
                With<TankTurret>,
                Without<Player>,
                Without<SnapshotReplica>,
                Without<SnapshotReplicaTurret>,
                Without<SnapshotReplicaBarrel>,
            ),
        >,
        Query<
            'static,
            'static,
            (
                &'static mut Transform,
                &'static mut TankBarrelState,
                &'static OwnedBy,
            ),
            (
                With<TankBarrel>,
                Without<Player>,
                Without<SnapshotReplica>,
                Without<SnapshotReplicaTurret>,
                Without<SnapshotReplicaBarrel>,
            ),
        >,
    )>,
    mut local_player_q: Query<
        (
            Entity,
            &mut Transform,
            Option<&mut Health>,
            Option<&TankParts>,
        ),
        (
            With<Player>,
            With<LocalPlayer>,
            Without<SnapshotReplica>,
            Without<TankTurret>,
            Without<TankBarrel>,
        ),
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
    let Snapshot {
        tick,
        entities,
        events,
    } = snapshot;
    if let Some(last_applied_tick) = snapshot_state.last_applied_tick
        && !is_snapshot_tick_newer(tick, last_applied_tick)
    {
        return;
    }
    snapshot_state.last_applied_tick = Some(tick);

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
    let mut impact_sync_state = impact_sync_state;
    let obstacle_by_net_id: HashMap<u64, Entity> = obstacle_q
        .iter()
        .map(|(entity, obstacle_id, _)| (obstacle_id.0, entity))
        .collect();
    let obstacle_world_positions: HashMap<Entity, Vec3> = obstacle_q
        .iter()
        .map(|(entity, _, tf)| (entity, tf.translation()))
        .collect();

    let mut seen_ids = HashSet::new();
    for entity_snapshot in entities {
        if local_session == Some(entity_snapshot.id.0) {
            if !matches!(run_mode.0, RunMode::Host) {
                let mut local_present = false;
                if let Some((local_entity, mut local_tf, maybe_health, tank_parts)) =
                    local_player_q.iter_mut().next()
                {
                    local_present = true;
                    let server_translation = Vec3::new(
                        entity_snapshot.position[0],
                        entity_snapshot.position[1],
                        entity_snapshot.position[2],
                    );
                    let server_rotation = normalized_rotation(entity_snapshot.rotation);
                    local_tf.translation = server_translation;
                    local_tf.rotation = server_rotation;
                    if let (Some([current, max]), Some(mut health)) =
                        (entity_snapshot.health, maybe_health)
                    {
                        health.max = max.max(0.0);
                        health.current = current.clamp(0.0, health.max);
                    }

                    if let (Some(parts), Some(server_turret_yaw)) =
                        (tank_parts, entity_snapshot.turret_yaw)
                        && let Ok((mut turret_tf, mut turret_state, owned_by)) =
                            transform_sets.p3().get_mut(parts.turret)
                        && owned_by.entity == local_entity
                    {
                        let synced_yaw = normalize_signed_angle(server_turret_yaw);
                        turret_state.yaw = synced_yaw;
                        turret_state.yaw_target = synced_yaw;
                        turret_state.yaw_velocity = 0.0;
                        turret_state.initialized = true;
                        turret_tf.rotation = Quat::from_rotation_y(synced_yaw);
                    }

                    if let (Some(parts), Some(server_barrel_pitch)) =
                        (tank_parts, entity_snapshot.barrel_pitch)
                        && let Ok((mut barrel_tf, mut barrel_state, owned_by)) =
                            transform_sets.p4().get_mut(parts.barrel)
                        && owned_by.entity == local_entity
                    {
                        let synced_pitch = server_barrel_pitch;
                        barrel_state.pitch = synced_pitch;
                        barrel_state.pitch_target = synced_pitch;
                        barrel_state.pitch_velocity = 0.0;
                        barrel_state.initialized = true;
                        barrel_tf.rotation = Quat::from_rotation_x(synced_pitch);
                    }
                }

                if !local_present && let Some(template) = player_template.as_deref() {
                    spawn_player_from_template(&mut commands, template, &physics_settings);
                    eprintln!(
                        "[net-client] local player respawn requested from snapshot: session_id={}",
                        entity_snapshot.id.0
                    );
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
            if let Ok(mut tf) = transform_sets.p0().get_mut(replica.root) {
                tf.translation = next_translation;
                tf.rotation = next_rotation;
            }
            if let (Some(turret_entity), Some(turret_yaw)) =
                (replica.turret, entity_snapshot.turret_yaw)
                && let Ok(mut turret_tf) = transform_sets.p1().get_mut(turret_entity)
            {
                turret_tf.rotation = Quat::from_rotation_y(turret_yaw);
            }
            if let (Some(barrel_entity), Some(barrel_pitch)) =
                (replica.barrel, entity_snapshot.barrel_pitch)
                && let Ok(mut barrel_tf) = transform_sets.p2().get_mut(barrel_entity)
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

    for event in events {
        match event {
            ServerEventDto::VehicleDespawned { id } => {
                if let Some(replica) = snapshot_state.by_net_id.remove(&id) {
                    commands.entity(replica.root).despawn_children().despawn();
                }
                if !matches!(run_mode.0, RunMode::Host)
                    && local_session == Some(id.0)
                    && let Ok((local_entity, _, _, _)) = local_player_q.single_mut()
                {
                    commands.entity(local_entity).despawn_children().despawn();
                }
            }
            ServerEventDto::VehicleSpawned { .. } => {}
            ServerEventDto::SessionAnnounce { .. } => {}
            ServerEventDto::ObstacleImpact {
                obstacle_id,
                point,
                normal,
                damage,
                impact_seq,
            } => {
                if matches!(run_mode.0, RunMode::Host) {
                    continue;
                }

                if let Some(sync_state) = impact_sync_state.as_deref_mut()
                    && !register_client_impact_seq(sync_state, impact_seq)
                {
                    continue;
                }

                let impact_point = Vec3::new(point[0], point[1], point[2]);
                let Some(target) = resolve_obstacle_target_for_impact(
                    obstacle_id,
                    impact_point,
                    &obstacle_by_net_id,
                    &obstacle_world_positions,
                ) else {
                    continue;
                };

                impact_events.write(ImpactEvent {
                    source: None,
                    target,
                    point: impact_point,
                    normal: Vec3::new(normal[0], normal[1], normal[2]),
                    damage,
                });
            }
        }
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
                Collider::cuboid(
                    template.collider_half_extents.x,
                    template.collider_half_extents.y,
                    template.collider_half_extents.z,
                ),
                Sensor,
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
            Collider::cuboid(0.80, 0.37, 1.10),
            Sensor,
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

fn normalize_signed_angle(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

fn register_client_impact_seq(state: &mut ClientImpactSyncState, seq: u32) -> bool {
    if !state.seen_seqs.insert(seq) {
        return false;
    }
    state.seq_order.push_back(seq);

    while state.seq_order.len() > CLIENT_IMPACT_SEQ_WINDOW {
        let Some(old_seq) = state.seq_order.pop_front() else {
            break;
        };
        state.seen_seqs.remove(&old_seq);
    }

    true
}

fn is_snapshot_tick_newer(new_tick: u32, current_tick: u32) -> bool {
    new_tick != current_tick && (new_tick.wrapping_sub(current_tick) as i32) > 0
}

fn compute_packet_loss_pct(pings_sent: u32, pongs_received: u32) -> Option<f32> {
    if pings_sent == 0 {
        return None;
    }
    let delivered = pongs_received.min(pings_sent) as f32;
    let sent = pings_sent as f32;
    let loss = (1.0 - delivered / sent).clamp(0.0, 1.0) * 100.0;
    Some(loss)
}

fn resolve_obstacle_target_for_impact(
    obstacle_id: u64,
    impact_point: Vec3,
    obstacle_by_net_id: &HashMap<u64, Entity>,
    obstacle_world_positions: &HashMap<Entity, Vec3>,
) -> Option<Entity> {
    const MAX_ID_MATCH_DISTANCE_SQ: f32 = 36.0;

    if let Some(mapped) = obstacle_by_net_id.get(&obstacle_id).copied()
        && obstacle_world_positions
            .get(&mapped)
            .is_some_and(|pos| pos.distance_squared(impact_point) <= MAX_ID_MATCH_DISTANCE_SQ)
    {
        return Some(mapped);
    }

    obstacle_world_positions
        .iter()
        .min_by(|(_, left_pos), (_, right_pos)| {
            left_pos
                .distance_squared(impact_point)
                .total_cmp(&right_pos.distance_squared(impact_point))
        })
        .map(|(entity, _)| *entity)
}
