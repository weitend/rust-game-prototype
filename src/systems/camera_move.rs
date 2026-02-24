use bevy::prelude::*;

use crate::{
    components::{
        aim_marker::AimMarker,
        follow_camera::FollowCamera,
        player::Player,
        tank::{TankBarrel, TankBarrelState, TankTurret},
    },
    resources::{
        aim_settings::{AimModeState, AimSettings},
        tank_settings::TankSettings,
    },
};

pub fn camera_move_system(
    player_q: Query<&Transform, (With<Player>, Without<FollowCamera>)>,
    turret_q: Query<&GlobalTransform, With<TankTurret>>,
    barrel_state_q: Query<&TankBarrelState, With<TankBarrel>>,
    marker_q: Query<(&Transform, &Visibility), (With<AimMarker>, Without<FollowCamera>)>,
    mut cam_q: Query<&mut Transform, (With<FollowCamera>, Without<Player>, Without<AimMarker>)>,
    aim_mode: Res<AimModeState>,
    aim_settings: Res<AimSettings>,
    tank_settings: Res<TankSettings>,
    time: Res<Time>,
) {
    let Ok(player) = player_q.single() else {
        return;
    };
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };

    let pivot_y = 0.5 + (player.translation.y - 0.5) * tank_settings.camera_follow_y;
    let pivot = Vec3::new(player.translation.x, pivot_y, player.translation.z);

    let (forward, right) = if let Ok(turret_tf) = turret_q.single() {
        let (_, turret_rotation, _) = turret_tf.to_scale_rotation_translation();
        (turret_rotation * -Vec3::Z, turret_rotation * Vec3::X)
    } else {
        (player.rotation * -Vec3::Z, player.rotation * Vec3::X)
    };

    let (target_pos, look_target, smooth) = if aim_mode.artillery_active {
        let pitch_t = if let Ok(barrel_state) = barrel_state_q.single() {
            let artillery_pitch_max = aim_settings.artillery_pitch_limit();
            let denom = (artillery_pitch_max - aim_settings.artillery_pitch_min).max(0.0001);
            ((barrel_state.pitch - aim_settings.artillery_pitch_min) / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let artillery_pivot = marker_q
            .single()
            .ok()
            .and_then(|(marker_tf, marker_visibility)| {
                if matches!(*marker_visibility, Visibility::Hidden) {
                    None
                } else {
                    Some(marker_tf.translation)
                }
            })
            .unwrap_or(pivot);

        let dynamic_back = aim_settings.artillery_camera_back
            + aim_settings.artillery_camera_back_pitch_extra * pitch_t;
        let dynamic_height = aim_settings.artillery_camera_height
            + aim_settings.artillery_camera_height_pitch_extra * pitch_t;
        let artillery_pos = artillery_pivot - forward * dynamic_back + Vec3::Y * dynamic_height;
        let artillery_look = artillery_pivot
            + Vec3::Y * aim_settings.artillery_camera_look_up
            + forward * aim_settings.artillery_camera_look_forward;
        (
            artillery_pos,
            artillery_look,
            aim_settings.artillery_camera_smooth,
        )
    } else {
        let pos = pivot
            + right * tank_settings.camera_offset_right
            + Vec3::Y * tank_settings.camera_offset_up
            - forward * tank_settings.camera_offset_back;
        let look = pivot
            + right * tank_settings.camera_look_right
            + Vec3::Y * tank_settings.camera_look_up
            + forward * tank_settings.camera_look_forward;
        (pos, look, tank_settings.camera_smooth)
    };

    let t = 1.0 - (-smooth * time.delta_secs()).exp();
    cam.translation = cam.translation.lerp(target_pos, t);
    cam.look_at(look_target, Vec3::Y);
}
