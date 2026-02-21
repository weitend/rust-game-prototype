use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::{
    components::{
        player::Player,
        tank::{TankBarrel, TankBarrelState, TankHull, TankTurret, TankTurretState},
    },
    resources::tank_settings::TankSettings,
};

pub fn tank_turret_yaw_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    settings: Res<TankSettings>,
    hull_q: Query<&Transform, (With<Player>, With<TankHull>, Without<TankTurret>)>,
    mut turret_q: Query<
        (&mut Transform, &mut TankTurretState),
        (With<TankTurret>, Without<Player>, Without<TankHull>),
    >,
) {
    let Ok(hull_tf) = hull_q.single() else {
        return;
    };

    let mut delta_x = 0.0;
    for event in mouse_motion.read() {
        delta_x += event.delta.x;
    }

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
    let forward = rotation * -Vec3::Z;
    forward.x.atan2(-forward.z)
}

pub fn tank_barrel_pitch_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    settings: Res<TankSettings>,
    mut barrel_q: Query<(&mut Transform, &mut TankBarrelState), With<TankBarrel>>,
) {
    let mut delta_y = 0.0;
    for event in mouse_motion.read() {
        delta_y += event.delta.y;
    }

    if delta_y.abs() <= f32::EPSILON {
        return;
    }

    let Ok((mut barrel_tf, mut barrel_state)) = barrel_q.single_mut() else {
        return;
    };

    let pitch_delta = -delta_y * settings.barrel_pitch_sensitivity;
    let next_pitch = barrel_state.pitch + pitch_delta;
    barrel_state.pitch = next_pitch.clamp(settings.barrel_pitch_min, settings.barrel_pitch_max);
    barrel_tf.rotation = Quat::from_rotation_x(barrel_state.pitch);
}
