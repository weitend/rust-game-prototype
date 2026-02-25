use bevy::prelude::*;

use crate::{
    components::{
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankParts, TankTurret, TankTurretState},
    },
    resources::{
        aim_settings::AimSettings,
        local_player::LocalPlayerContext,
        run_mode::{AppRunMode, RunMode},
        tank_settings::TankSettings,
    },
    utils::local_player::resolve_local_player_entity,
};

const ANGLE_EPSILON: f32 = 1.0e-4;
const ANGULAR_VEL_EPSILON: f32 = 1.0e-3;

pub fn tank_turret_yaw_system(
    time: Res<Time>,
    settings: Res<TankSettings>,
    run_mode: Res<AppRunMode>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut player_q: Query<(Entity, &mut PlayerIntent, &TankParts, Option<&LocalPlayer>), With<Player>>,
    mut turret_q: Query<
        (&mut Transform, &mut TankTurretState, &OwnedBy),
        (With<TankTurret>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();

    if matches!(run_mode.0, RunMode::Server | RunMode::Host) {
        for (player_entity, mut intent, tank_parts, local_marker) in &mut player_q {
            apply_turret_yaw_for_player(
                player_entity,
                &intent,
                tank_parts,
                dt,
                &settings,
                &mut turret_q,
            );
            if local_marker.is_none() {
                intent.turret_yaw_delta = 0.0;
            }
        }
        return;
    }

    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((_, intent, tank_parts, _)) = player_q.get(player_entity) else {
        return;
    };
    apply_turret_yaw_for_player(
        player_entity,
        &intent,
        tank_parts,
        dt,
        &settings,
        &mut turret_q,
    );
}

fn normalize_angle(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

fn yaw_from_rotation(rotation: Quat) -> f32 {
    let (yaw, _, _) = rotation.to_euler(EulerRot::YXZ);
    normalize_angle(yaw)
}

pub fn tank_barrel_pitch_system(
    time: Res<Time>,
    aim_settings: Res<AimSettings>,
    settings: Res<TankSettings>,
    run_mode: Res<AppRunMode>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut player_q: Query<(Entity, &mut PlayerIntent, &TankParts, Option<&LocalPlayer>), With<Player>>,
    mut barrel_q: Query<
        (&mut Transform, &mut TankBarrelState, &OwnedBy),
        (With<TankBarrel>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();

    if matches!(run_mode.0, RunMode::Server | RunMode::Host) {
        for (player_entity, mut intent, tank_parts, local_marker) in &mut player_q {
            apply_barrel_pitch_for_player(
                player_entity,
                &intent,
                tank_parts,
                dt,
                &settings,
                &aim_settings,
                &mut barrel_q,
            );
            if local_marker.is_none() {
                intent.barrel_pitch_delta = 0.0;
            }
        }
        return;
    }

    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((_, intent, tank_parts, _)) = player_q.get(player_entity) else {
        return;
    };
    apply_barrel_pitch_for_player(
        player_entity,
        &intent,
        tank_parts,
        dt,
        &settings,
        &aim_settings,
        &mut barrel_q,
    );
}

fn pitch_from_rotation(rotation: Quat) -> f32 {
    let (_, pitch, _) = rotation.to_euler(EulerRot::YXZ);
    pitch
}

fn apply_turret_yaw_for_player(
    player_entity: Entity,
    intent: &PlayerIntent,
    tank_parts: &TankParts,
    dt: f32,
    settings: &TankSettings,
    turret_q: &mut Query<
        (&mut Transform, &mut TankTurretState, &OwnedBy),
        (With<TankTurret>, Without<Player>),
    >,
) {
    let delta_x = intent.turret_yaw_delta;

    let Ok((mut turret_tf, mut turret_state, owned_by)) = turret_q.get_mut(tank_parts.turret)
    else {
        return;
    };
    if owned_by.entity != player_entity {
        warn!(
            "TankTurret {:?} is owned by {:?}, expected {:?}",
            tank_parts.turret, owned_by.entity, player_entity
        );
        return;
    };

    if !turret_state.initialized {
        let local_yaw = yaw_from_rotation(turret_tf.rotation);
        turret_state.yaw = local_yaw;
        turret_state.yaw_target = local_yaw;
        turret_state.yaw_velocity = 0.0;
        turret_state.initialized = true;
    }

    let yaw_delta = -delta_x * settings.turret_yaw_sensitivity;
    turret_state.yaw_target = (turret_state.yaw_target + yaw_delta)
        .clamp(-settings.turret_yaw_limit, settings.turret_yaw_limit);

    let yaw_error = turret_state.yaw_target - turret_state.yaw;
    let desired_velocity = (yaw_error * settings.turret_yaw_tracking_gain).clamp(
        -settings.turret_yaw_max_speed,
        settings.turret_yaw_max_speed,
    );
    let vel_delta = desired_velocity - turret_state.yaw_velocity;
    let vel_step = settings.turret_yaw_accel * dt;
    turret_state.yaw_velocity += vel_delta.clamp(-vel_step, vel_step);

    let yaw_damping = (1.0 - settings.turret_yaw_damping * dt).clamp(0.0, 1.0);
    turret_state.yaw_velocity *= yaw_damping;

    turret_state.yaw += turret_state.yaw_velocity * dt;
    let clamped_yaw = turret_state
        .yaw
        .clamp(-settings.turret_yaw_limit, settings.turret_yaw_limit);
    if (clamped_yaw - turret_state.yaw).abs() > ANGLE_EPSILON {
        if (clamped_yaw >= settings.turret_yaw_limit && turret_state.yaw_velocity > 0.0)
            || (clamped_yaw <= -settings.turret_yaw_limit && turret_state.yaw_velocity < 0.0)
        {
            turret_state.yaw_velocity = 0.0;
        }
    }
    turret_state.yaw = clamped_yaw;
    turret_state.yaw_target = turret_state
        .yaw_target
        .clamp(-settings.turret_yaw_limit, settings.turret_yaw_limit);

    if (turret_state.yaw_target - turret_state.yaw).abs() <= ANGLE_EPSILON
        && turret_state.yaw_velocity.abs() <= ANGULAR_VEL_EPSILON
    {
        turret_state.yaw = turret_state.yaw_target;
        turret_state.yaw_velocity = 0.0;
    }

    turret_tf.rotation = Quat::from_rotation_y(turret_state.yaw);
}

fn apply_barrel_pitch_for_player(
    player_entity: Entity,
    intent: &PlayerIntent,
    tank_parts: &TankParts,
    dt: f32,
    settings: &TankSettings,
    aim_settings: &AimSettings,
    barrel_q: &mut Query<
        (&mut Transform, &mut TankBarrelState, &OwnedBy),
        (With<TankBarrel>, Without<Player>),
    >,
) {
    let delta_y = intent.barrel_pitch_delta;

    let Ok((mut barrel_tf, mut barrel_state, owned_by)) = barrel_q.get_mut(tank_parts.barrel)
    else {
        return;
    };
    if owned_by.entity != player_entity {
        warn!(
            "TankBarrel {:?} is owned by {:?}, expected {:?}",
            tank_parts.barrel, owned_by.entity, player_entity
        );
        return;
    };

    barrel_state.artillery_mode_active = intent.artillery_active;

    let (pitch_min, pitch_max) = if intent.artillery_active {
        (
            aim_settings.artillery_pitch_min,
            aim_settings.artillery_pitch_limit(),
        )
    } else {
        (settings.barrel_pitch_min, settings.barrel_pitch_max)
    };

    if !barrel_state.initialized {
        let pitch = pitch_from_rotation(barrel_tf.rotation);
        barrel_state.pitch = pitch;
        barrel_state.pitch_target = pitch;
        barrel_state.pitch_velocity = 0.0;
        barrel_state.initialized = true;
    }

    if intent.artillery_active
        && barrel_state.pitch_target < pitch_min
        && delta_y.abs() <= f32::EPSILON
    {
        let raise = aim_settings.artillery_auto_raise_speed * dt;
        barrel_state.pitch_target = (barrel_state.pitch_target + raise).min(pitch_min);
    } else if delta_y.abs() > f32::EPSILON {
        let pitch_delta = -delta_y * settings.barrel_pitch_sensitivity;
        barrel_state.pitch_target += pitch_delta;
    }

    barrel_state.pitch_target = barrel_state.pitch_target.clamp(pitch_min, pitch_max);

    let pitch_error = barrel_state.pitch_target - barrel_state.pitch;
    let desired_velocity = (pitch_error * settings.barrel_pitch_tracking_gain).clamp(
        -settings.barrel_pitch_max_speed,
        settings.barrel_pitch_max_speed,
    );
    let vel_delta = desired_velocity - barrel_state.pitch_velocity;
    let vel_step = settings.barrel_pitch_accel * dt;
    barrel_state.pitch_velocity += vel_delta.clamp(-vel_step, vel_step);

    let pitch_damping = (1.0 - settings.barrel_pitch_damping * dt).clamp(0.0, 1.0);
    barrel_state.pitch_velocity *= pitch_damping;

    barrel_state.pitch += barrel_state.pitch_velocity * dt;
    let clamped_pitch = barrel_state.pitch.clamp(pitch_min, pitch_max);
    if (clamped_pitch - barrel_state.pitch).abs() > ANGLE_EPSILON {
        if (clamped_pitch >= pitch_max && barrel_state.pitch_velocity > 0.0)
            || (clamped_pitch <= pitch_min && barrel_state.pitch_velocity < 0.0)
        {
            barrel_state.pitch_velocity = 0.0;
        }
    }
    barrel_state.pitch = clamped_pitch;
    barrel_state.pitch_target = barrel_state.pitch_target.clamp(pitch_min, pitch_max);
    if (barrel_state.pitch_target - barrel_state.pitch).abs() <= ANGLE_EPSILON
        && barrel_state.pitch_velocity.abs() <= ANGULAR_VEL_EPSILON
    {
        barrel_state.pitch = barrel_state.pitch_target;
        barrel_state.pitch_velocity = 0.0;
    }
    barrel_tf.rotation = Quat::from_rotation_x(barrel_state.pitch);
}
