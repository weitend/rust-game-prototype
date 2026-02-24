use bevy::prelude::*;

use crate::{
    components::{
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankHull, TankParts, TankTurret, TankTurretState},
    },
    resources::{
        aim_settings::AimSettings, local_player::LocalPlayerContext, tank_settings::TankSettings,
    },
    utils::local_player::resolve_local_player_entity,
};

pub fn tank_turret_yaw_system(
    settings: Res<TankSettings>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_q: Query<(&PlayerIntent, &Transform, &TankParts), (With<Player>, With<TankHull>)>,
    mut turret_q: Query<
        (&mut Transform, &mut TankTurretState, &OwnedBy),
        (With<TankTurret>, Without<Player>, Without<TankHull>),
    >,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((intent, hull_tf, tank_parts)) = player_q.get(player_entity) else {
        return;
    };
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

    let yaw_delta = -delta_x * settings.turret_yaw_sensitivity;
    turret_state.yaw = normalize_angle(turret_state.yaw + yaw_delta);

    let hull_yaw = yaw_from_rotation(hull_tf.rotation);
    let local_yaw = normalize_angle(turret_state.yaw - hull_yaw)
        .clamp(-settings.turret_yaw_limit, settings.turret_yaw_limit);
    turret_state.yaw = normalize_angle(hull_yaw + local_yaw);
    turret_tf.rotation = Quat::from_rotation_y(local_yaw);
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
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_q: Query<(&PlayerIntent, &TankParts), With<Player>>,
    mut barrel_q: Query<
        (&mut Transform, &mut TankBarrelState, &OwnedBy),
        (With<TankBarrel>, Without<Player>),
    >,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((intent, tank_parts)) = player_q.get(player_entity) else {
        return;
    };
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

    if intent.artillery_active && barrel_state.pitch < pitch_min && delta_y.abs() <= f32::EPSILON {
        let raise = aim_settings.artillery_auto_raise_speed * time.delta_secs();
        barrel_state.pitch = (barrel_state.pitch + raise).min(pitch_min);
    } else if delta_y.abs() > f32::EPSILON {
        let pitch_delta = -delta_y * settings.barrel_pitch_sensitivity;
        barrel_state.pitch += pitch_delta;
    }

    barrel_state.pitch = barrel_state.pitch.clamp(pitch_min, pitch_max);
    barrel_tf.rotation = Quat::from_rotation_x(barrel_state.pitch);
}
