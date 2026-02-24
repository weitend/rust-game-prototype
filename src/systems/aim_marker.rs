use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        aim_marker::{AimMarker, ArtilleryVignette},
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankMuzzle, TankParts},
        weapon::HitscanWeapon,
    },
    resources::{aim_settings::AimSettings, local_player::LocalPlayerContext},
    utils::{
        ballistics::predict_ballistic_impact, local_player::resolve_local_player_entity,
        muzzle::muzzle_ray,
    },
};

pub fn update_aim_marker_system(
    mut marker_q: Query<(&mut Transform, &mut Visibility), With<AimMarker>>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_q: Query<(Entity, &TankParts, &HitscanWeapon, &PlayerIntent), With<Player>>,
    muzzle_q: Query<(&GlobalTransform, &OwnedBy), With<TankMuzzle>>,
    rapier_context: ReadRapierContext,
    settings: Res<AimSettings>,
) {
    let Some((mut marker_tf, mut marker_visibility)) = marker_q.iter_mut().next() else {
        return;
    };
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok((player_entity, tank_parts, weapon, intent)) = player_q.get(player_entity) else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok((muzzle_tf, owned_by)) = muzzle_q.get(tank_parts.muzzle) else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    if owned_by.entity != player_entity {
        warn!(
            "TankMuzzle {:?} is owned by {:?}, expected {:?}",
            tank_parts.muzzle, owned_by.entity, player_entity
        );
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok(rapier_context) = rapier_context.single() else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    let Some((ray_origin, ray_dir)) = muzzle_ray(muzzle_tf) else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    let filter = QueryFilter::new()
        .exclude_collider(player_entity)
        .exclude_rigid_body(player_entity)
        .exclude_sensors();
    let impact = if intent.artillery_active {
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
                    if n == Vec3::ZERO {
                        Vec3::Y
                    } else {
                        n
                    }
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
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_intent_q: Query<&PlayerIntent, With<Player>>,
    settings: Res<AimSettings>,
    mut vignette_q: Query<(&mut BorderColor, &mut BackgroundColor), With<ArtilleryVignette>>,
) {
    let Some((mut border_color, mut bg_color)) = vignette_q.iter_mut().next() else {
        return;
    };

    let artillery_active = resolve_local_player_entity(&local_player_ctx, &local_player_q)
        .and_then(|player_entity| player_intent_q.get(player_entity).ok())
        .map(|intent| intent.artillery_active)
        .unwrap_or(false);
    let alpha = if artillery_active {
        settings.vignette_alpha
    } else {
        0.0
    };

    *border_color = BorderColor::all(Color::srgba(0.02, 0.03, 0.04, alpha));
    bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha * 0.10);
}
