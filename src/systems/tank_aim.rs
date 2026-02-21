use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::{
    components::tank::{TankTurret, TankTurretState},
    resources::tank_settings::TankSettings,
};

pub fn tank_turret_yaw_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    settings: Res<TankSettings>,
    mut turret_q: Query<(&mut Transform, &mut TankTurretState), With<TankTurret>>,
) {
    let mut delta_x = 0.0;
    for event in mouse_motion.read() {
        delta_x += event.delta.x;
    }

    if delta_x.abs() <= f32::EPSILON {
        return;
    }

    let Ok((mut turret_tf, mut turret_state)) = turret_q.single_mut() else {
        return;
    };

    let yaw_delta = -delta_x * settings.turret_yaw_sensitivity;
    let next_yaw = normalize_angle(turret_state.yaw + yaw_delta);
    turret_state.yaw = next_yaw.clamp(-settings.turret_yaw_limit, settings.turret_yaw_limit);
    turret_tf.rotation = Quat::from_rotation_y(turret_state.yaw);
}

fn normalize_angle(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}
