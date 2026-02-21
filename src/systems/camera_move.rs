use bevy::prelude::*;

use crate::components::{follow_camera::FollowCamera, player::Player, tank::TankTurret};

const OFFSET_RIGHT: f32 = -1.0;
const OFFSET_UP: f32 = 1.8;
const OFFSET_BACK: f32 = 6.0;
const LOOK_RIGHT: f32 = 0.2;
const LOOK_UP: f32 = 0.2;
const LOOK_FORWARD: f32 = 1.6;
const FOLLOW_Y: f32 = 0.8;
const SMOOTH: f32 = 5.0;

pub fn camera_move_system(
    player_q: Query<&Transform, (With<Player>, Without<FollowCamera>)>,
    turret_q: Query<&GlobalTransform, With<TankTurret>>,
    mut cam_q: Query<&mut Transform, (With<FollowCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player) = player_q.single() else {
        return;
    };
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };

    let pivot_y = 0.5 + (player.translation.y - 0.5) * FOLLOW_Y;
    let pivot = Vec3::new(player.translation.x, pivot_y, player.translation.z);

    let (forward, right) = if let Ok(turret_tf) = turret_q.single() {
        let (_, turret_rotation, _) = turret_tf.to_scale_rotation_translation();
        (turret_rotation * -Vec3::Z, turret_rotation * Vec3::X)
    } else {
        (player.rotation * -Vec3::Z, player.rotation * Vec3::X)
    };

    let target_pos = pivot + right * OFFSET_RIGHT + Vec3::Y * OFFSET_UP - forward * OFFSET_BACK;

    let look_target = pivot + right * LOOK_RIGHT + Vec3::Y * LOOK_UP + forward * LOOK_FORWARD;

    let t = 1.0 - (-SMOOTH * time.delta_secs()).exp();
    cam.translation = cam.translation.lerp(target_pos, t);
    cam.look_at(look_target, Vec3::Y);
}
