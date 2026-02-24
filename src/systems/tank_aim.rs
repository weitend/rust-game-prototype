use bevy::prelude::*;

use crate::{
    components::{
        intent::PlayerIntent,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankHull, TankTurret, TankTurretState},
    },
    resources::{
        aim_settings::{AimModeState, AimSettings},
        tank_settings::TankSettings,
    },
};

pub fn update_aim_mode_system(
    mut aim_mode: ResMut<AimModeState>,
    player_intent_q: Query<&PlayerIntent, (With<Player>, With<LocalPlayer>)>,
) {
    aim_mode.artillery_active = player_intent_q
        .single()
        .map(|intent| intent.artillery_active)
        .unwrap_or(false);
}

pub fn tank_turret_yaw_system(
    settings: Res<TankSettings>,
    player_intent_q: Query<&PlayerIntent, (With<Player>, With<LocalPlayer>)>,
    hull_q: Query<&Transform, (With<Player>, With<TankHull>, Without<TankTurret>)>,
    mut turret_q: Query<
        (&mut Transform, &mut TankTurretState),
        (With<TankTurret>, Without<Player>, Without<TankHull>),
    >,
) {
    let Ok(intent) = player_intent_q.single() else {
        return;
    };
    let Ok(hull_tf) = hull_q.single() else {
        return;
    };
    let delta_x = intent.turret_yaw_delta;

    let Ok((mut turret_tf, mut turret_state)) = turret_q.single_mut() else {
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
    player_intent_q: Query<&PlayerIntent, (With<Player>, With<LocalPlayer>)>,
    mut barrel_q: Query<(&mut Transform, &mut TankBarrelState), With<TankBarrel>>,
) {
    let Ok(intent) = player_intent_q.single() else {
        return;
    };
    let delta_y = intent.barrel_pitch_delta;

    let Ok((mut barrel_tf, mut barrel_state)) = barrel_q.single_mut() else {
        return;
    };

    let (pitch_min, pitch_max) = if intent.artillery_active {
        (
            aim_settings.artillery_pitch_min,
            aim_settings.artillery_pitch_limit(),
        )
    } else {
        (settings.barrel_pitch_min, settings.barrel_pitch_max)
    };

    if intent.artillery_active && barrel_state.pitch < pitch_min && delta_y.abs() <= f32::EPSILON
    {
        let raise = aim_settings.artillery_auto_raise_speed * time.delta_secs();
        barrel_state.pitch = (barrel_state.pitch + raise).min(pitch_min);
    } else if delta_y.abs() > f32::EPSILON {
        let pitch_delta = -delta_y * settings.barrel_pitch_sensitivity;
        barrel_state.pitch += pitch_delta;
    }

    barrel_state.pitch = barrel_state.pitch.clamp(pitch_min, pitch_max);
    barrel_tf.rotation = Quat::from_rotation_x(barrel_state.pitch);
}
