use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        aim_marker::AimMarker,
        player::{LocalPlayer, Player},
        tank::TankMuzzle,
        weapon::HitscanWeapon,
    },
    resources::aim_settings::AimSettings,
};

pub fn update_aim_marker_system(
    mut marker_q: Query<(&mut Transform, &mut Visibility), With<AimMarker>>,
    local_player_q: Query<(Entity, &HitscanWeapon), (With<Player>, With<LocalPlayer>)>,
    muzzle_q: Query<&GlobalTransform, With<TankMuzzle>>,
    rapier_context: ReadRapierContext,
    settings: Res<AimSettings>,
) {
    let Ok((mut marker_tf, mut marker_visibility)) = marker_q.single_mut() else {
        return;
    };
    let Ok((player_entity, weapon)) = local_player_q.single() else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok(muzzle_tf) = muzzle_q.single() else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok(rapier_context) = rapier_context.single() else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    let ray_origin = muzzle_tf.translation();
    let (_, muzzle_rotation, _) = muzzle_tf.to_scale_rotation_translation();
    let ray_dir = (muzzle_rotation * -Vec3::Z).normalize_or_zero();
    if ray_dir == Vec3::ZERO {
        *marker_visibility = Visibility::Hidden;
        return;
    }

    let filter = QueryFilter::new()
        .exclude_collider(player_entity)
        .exclude_rigid_body(player_entity)
        .exclude_sensors();
    let max_distance = weapon.range.max(settings.range_fallback);
    let Some((_, hit)) =
        rapier_context.cast_ray_and_get_normal(ray_origin, ray_dir, max_distance, true, filter)
    else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    let normal = {
        let normalized = hit.normal.normalize_or_zero();
        if normalized == Vec3::ZERO {
            Vec3::Y
        } else {
            normalized
        }
    };

    marker_tf.translation = hit.point + normal * settings.marker_surface_offset;
    marker_tf.rotation = Quat::from_rotation_arc(Vec3::Y, normal);
    *marker_visibility = Visibility::Visible;
}
