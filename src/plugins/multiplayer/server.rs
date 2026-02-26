use std::{collections::HashSet, io::ErrorKind, net::SocketAddr};

use bevy::prelude::*;

use crate::{
    components::{
        combat::Health,
        intent::PlayerIntent,
        obstacle::ObstacleNetId,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankParts, TankTurret, TankTurretState},
    },
    network::protocol::{
        ClientPacket, EntitySnapshot, NetEntityId, ServerEventDto, ServerPacket, Snapshot,
    },
    resources::{
        player_motion_settings::PlayerMotionSettings,
        player_physics_settings::PlayerPhysicsSettings,
        run_mode::{AppRunMode, RunMode},
    },
    systems::impact::ImpactEvent,
};

use super::{
    config::NetConfig,
    intent::player_intent_from_client_input,
    io::send_server_packet,
    spawn::{
        assign_player_entity_for_session, despawn_network_owned_player, find_unbound_local_player,
        first_unbound_local_player_entity,
    },
    state::{
        HostLoopbackNonce, NetLifecycleMessage, NetworkControlledPlayer, ServerNetState,
        ServerSession,
    },
};

pub(super) fn server_receive_packets(
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
                            was_present_last_snapshot: false,
                            respawn_deadline_secs: None,
                            is_host_local: bind_to_host_local,
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

                    if let Some(last_seq) = session.last_input_seq {
                        if !is_sequence_newer(input.seq, last_seq) {
                            continue;
                        }
                    }
                    session.last_input_seq = Some(input.seq);

                    if matches!(run_mode.0, RunMode::Host) && session.is_host_local {
                        continue;
                    }

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
                        session.was_present_last_snapshot = false;
                        session.respawn_deadline_secs = None;
                        entity
                    };

                    if let Ok(mut intent) = player_intent_q.get_mut(player_entity) {
                        *intent = player_intent_from_client_input(&input);
                        if input.seq % 20 == 0 || input.fire_just_pressed {
                            eprintln!(
                                "[net-server] input applied: session_id={} entity={:?} seq={} throttle={:.2} turn={:.2} yaw_d={:.4} pitch_d={:.4} artillery={} fire_pressed={} fire_just_pressed={}",
                                session.id,
                                player_entity,
                                input.seq,
                                input.throttle,
                                input.turn,
                                input.turret_yaw_delta,
                                input.barrel_pitch_delta,
                                input.artillery_active,
                                input.fire_pressed,
                                input.fire_just_pressed
                            );
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn server_respawn_missing_players(
    mut commands: Commands,
    state: Option<ResMut<ServerNetState>>,
    time: Res<Time>,
    run_mode: Res<AppRunMode>,
    motion_settings: Res<PlayerMotionSettings>,
    physics_settings: Res<PlayerPhysicsSettings>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    network_player_q: Query<(), With<NetworkControlledPlayer>>,
) {
    let Some(mut state) = state else {
        return;
    };

    const RESPAWN_DELAY_SECS: f64 = 2.5;
    let now_secs = time.elapsed_secs_f64();
    let mut reserved_local_players = HashSet::new();
    if matches!(run_mode.0, RunMode::Host) {
        for session in state.sessions.values() {
            if let Some(entity) = session.player_entity
                && local_player_q.get(entity).is_ok()
            {
                reserved_local_players.insert(entity);
            }
        }
    }

    for session in state.sessions.values_mut() {
        let is_alive = session.player_entity.is_some_and(|entity| {
            network_player_q.get(entity).is_ok() || local_player_q.get(entity).is_ok()
        });
        if is_alive {
            session.respawn_deadline_secs = None;
            continue;
        }

        if matches!(run_mode.0, RunMode::Host) && session.is_host_local {
            if let Some(local_entity) =
                find_unbound_local_player(&local_player_q, &reserved_local_players)
            {
                session.player_entity = Some(local_entity);
                session.was_present_last_snapshot = false;
                session.respawn_deadline_secs = None;
                reserved_local_players.insert(local_entity);
                continue;
            }
        }

        let respawn_deadline = *session
            .respawn_deadline_secs
            .get_or_insert(now_secs + RESPAWN_DELAY_SECS);
        if now_secs < respawn_deadline {
            continue;
        }

        let preferred_host_local = if matches!(run_mode.0, RunMode::Host) && session.is_host_local {
            find_unbound_local_player(&local_player_q, &reserved_local_players)
        } else {
            None
        };
        if let Some(local_entity) = preferred_host_local {
            reserved_local_players.insert(local_entity);
        }

        let player_entity = assign_player_entity_for_session(
            &mut commands,
            &run_mode,
            preferred_host_local,
            &motion_settings,
            &physics_settings,
            session.id,
        );
        session.player_entity = Some(player_entity);
        session.was_present_last_snapshot = false;
        session.respawn_deadline_secs = None;
    }
}

pub(super) fn server_send_snapshots(
    state: Option<ResMut<ServerNetState>>,
    time: Res<Time>,
    mut impact_events: MessageReader<ImpactEvent>,
    player_snapshot_q: Query<(&Transform, Option<&Health>, Option<&TankParts>)>,
    turret_state_q: Query<&TankTurretState, With<TankTurret>>,
    barrel_state_q: Query<&TankBarrelState, With<TankBarrel>>,
    obstacle_id_q: Query<&ObstacleNetId>,
    owned_by_q: Query<&OwnedBy>,
) {
    let Some(mut state) = state else {
        return;
    };

    state.snapshot_timer.tick(time.delta());
    if !state.snapshot_timer.just_finished() {
        return;
    }

    let mut entities = Vec::new();
    let mut events = Vec::new();
    for impact in impact_events.read() {
        let obstacle_id = obstacle_id_q.get(impact.target).ok().copied().or_else(|| {
            owned_by_q
                .get(impact.target)
                .ok()
                .and_then(|owner| obstacle_id_q.get(owner.entity).ok().copied())
        });
        let Some(obstacle_id) = obstacle_id else {
            continue;
        };

        let impact_seq = state.next_impact_seq;
        state.next_impact_seq = state.next_impact_seq.wrapping_add(1);
        events.push(ServerEventDto::ObstacleImpact {
            obstacle_id: obstacle_id.0,
            point: [impact.point.x, impact.point.y, impact.point.z],
            normal: [impact.normal.x, impact.normal.y, impact.normal.z],
            damage: impact.damage,
            impact_seq,
        });
    }
    for session in state.sessions.values_mut() {
        let net_id = NetEntityId(session.id);
        let Some(player_entity) = session.player_entity else {
            if session.was_present_last_snapshot {
                events.push(ServerEventDto::VehicleDespawned { id: net_id });
            }
            session.was_present_last_snapshot = false;
            continue;
        };
        let Ok((tf, health, tank_parts)) = player_snapshot_q.get(player_entity) else {
            if session.was_present_last_snapshot {
                events.push(ServerEventDto::VehicleDespawned { id: net_id });
            }
            session.was_present_last_snapshot = false;
            continue;
        };

        if !session.was_present_last_snapshot {
            events.push(ServerEventDto::VehicleSpawned { id: net_id });
        }
        session.was_present_last_snapshot = true;

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
            id: net_id,
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
        events,
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

pub(super) fn server_prune_stale_sessions(
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

pub(super) fn server_log_controlled_players(
    players: Query<(&Transform, &NetworkControlledPlayer), With<Player>>,
) {
    for (tf, network_owner) in &players {
        eprintln!(
            "[net-server] sim: session_id={} position=({:.2}, {:.2}, {:.2})",
            network_owner.session_id, tf.translation.x, tf.translation.y, tf.translation.z
        );
    }
}

fn is_sequence_newer(new_seq: u32, current_seq: u32) -> bool {
    new_seq != current_seq && (new_seq.wrapping_sub(current_seq) as i32) > 0
}
