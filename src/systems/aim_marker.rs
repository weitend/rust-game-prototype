use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        aim_marker::{AimMarker, ArtilleryVignette},
        player::{LocalPlayer, Player},
        tank::TankMuzzle,
        weapon::HitscanWeapon,
    },
    resources::aim_settings::{AimModeState, AimSettings},
    utils::ballistics::predict_ballistic_impact,
};

pub fn update_aim_marker_system(
    mut marker_q: Query<(&mut Transform, &mut Visibility), With<AimMarker>>,
    local_player_q: Query<(Entity, &HitscanWeapon), (With<Player>, With<LocalPlayer>)>,
    muzzle_q: Query<&GlobalTransform, With<TankMuzzle>>,
    rapier_context: ReadRapierContext,
    aim_mode: Res<AimModeState>,
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
    let impact = if aim_mode.artillery_active {
        predict_ballistic_impact(
            &rapier_context,
            ray_origin,
            ray_dir,
            settings.artillery_ballistic_params(weapon.range),
            filter,
        )
    } else {
        rapier_context
            .cast_ray_and_get_normal(
                ray_origin,
                ray_dir,
                weapon.range.max(settings.range_fallback),
                true,
                filter,
            )
            .map(|(target, hit)| crate::utils::ballistics::BallisticImpact {
                target,
                point: hit.point,
                normal: {
                    let n = hit.normal.normalize_or_zero();
                    if n == Vec3::ZERO { Vec3::Y } else { n }
                },
                travel_distance: hit.time_of_impact.max(0.0),
            })
    };

    let Some(impact) = impact else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    marker_tf.translation = impact.point + impact.normal * settings.marker_surface_offset;
    marker_tf.rotation = Quat::from_rotation_arc(Vec3::Y, impact.normal);
    *marker_visibility = Visibility::Visible;
}

pub fn update_artillery_vignette_system(
    aim_mode: Res<AimModeState>,
    settings: Res<AimSettings>,
    mut vignette_q: Query<(&mut BorderColor, &mut BackgroundColor), With<ArtilleryVignette>>,
) {
    let Ok((mut border_color, mut bg_color)) = vignette_q.single_mut() else {
        return;
    };

    let alpha = if aim_mode.artillery_active {
        settings.vignette_alpha
    } else {
        0.0
    };

    *border_color = BorderColor::all(Color::srgba(0.02, 0.03, 0.04, alpha));
    bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha * 0.10);
}
