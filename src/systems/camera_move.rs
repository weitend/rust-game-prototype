use bevy::prelude::*;

use crate::{
    components::{
        aim_marker::AimMarker,
        follow_camera::FollowCamera,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankParts, TankTurret, TankTurretState},
    },
    resources::{
        aim_settings::AimSettings, local_player::LocalPlayerContext, tank_settings::TankSettings,
    },
    utils::local_player::resolve_local_player_entity,
};

pub fn camera_move_system(
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_q: Query<(&Transform, &TankParts), (With<Player>, Without<FollowCamera>)>,
    turret_q: Query<(&TankTurretState, &OwnedBy), With<TankTurret>>,
    barrel_q: Query<(&TankBarrelState, &OwnedBy), With<TankBarrel>>,
    marker_q: Query<(&Transform, &Visibility), (With<AimMarker>, Without<FollowCamera>)>,
    mut cam_q: Query<&mut Transform, (With<FollowCamera>, Without<Player>, Without<AimMarker>)>,
    aim_settings: Res<AimSettings>,
    tank_settings: Res<TankSettings>,
    time: Res<Time>,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((player, tank_parts)) = player_q.get(player_entity) else {
        return;
    };
    let Some(mut cam) = cam_q.iter_mut().next() else {
        return;
    };

    let pivot_y = 0.5 + (player.translation.y - 0.5) * tank_settings.camera_follow_y;
    let pivot = Vec3::new(player.translation.x, pivot_y, player.translation.z);
    let (forward, right) = if let Ok((turret_state, owned_by)) = turret_q.get(tank_parts.turret) {
        if owned_by.entity != player_entity {
            warn!(
                "TankTurret {:?} is owned by {:?}, expected {:?}",
                tank_parts.turret, owned_by.entity, player_entity
            );
            (player.rotation * -Vec3::Z, player.rotation * Vec3::X)
        } else {
            let aim_rotation = player.rotation * Quat::from_rotation_y(turret_state.yaw_target);
            (aim_rotation * -Vec3::Z, aim_rotation * Vec3::X)
        }
    } else {
        (player.rotation * -Vec3::Z, player.rotation * Vec3::X)
    };
    let barrel_state = if let Ok((barrel_state, owned_by)) = barrel_q.get(tank_parts.barrel) {
        if owned_by.entity != player_entity {
            warn!(
                "TankBarrel {:?} is owned by {:?}, expected {:?}",
                tank_parts.barrel, owned_by.entity, player_entity
            );
            None
        } else {
            Some(barrel_state)
        }
    } else {
        None
    };
    let artillery_active = barrel_state
        .map(|state| state.artillery_mode_active)
        .unwrap_or(false);

    let (target_pos, look_target, smooth) = if artillery_active {
        let artillery_pitch_max = aim_settings.artillery_pitch_limit();
        let denom = (artillery_pitch_max - aim_settings.artillery_pitch_min).max(0.0001);
        let pitch_t = if let Some(barrel_state) = barrel_state {
            ((barrel_state.pitch_target - aim_settings.artillery_pitch_min) / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let artillery_pivot = marker_q
            .iter()
            .next()
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
