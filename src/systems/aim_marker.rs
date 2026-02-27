use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        aim_marker::{AimMarker, ArtilleryVignette},
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        tank::{TankBarrel, TankBarrelState, TankMuzzle, TankParts, TankTurret},
        weapon::{HitscanWeapon, ProjectileWeaponProfile},
    },
    resources::run_mode::{AppRunMode, RunMode},
    resources::{aim_settings::AimSettings, local_player::LocalPlayerContext},
    utils::{
        ballistics::{BallisticParams, cast_hitscan_impact, predict_ballistic_impact},
        local_player::resolve_local_player_entity,
        muzzle::muzzle_ray_from_local_hierarchy,
        weapon_ballistics::{ProjectileSpawnParams, build_projectile_spawn_params},
    },
};

pub fn update_aim_marker_system(
    mut marker_q: Query<(&mut Transform, &mut Visibility), With<AimMarker>>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_q: Query<
        (
            Entity,
            &Transform,
            &TankParts,
            &HitscanWeapon,
            Option<&ProjectileWeaponProfile>,
        ),
        (With<Player>, Without<AimMarker>),
    >,
    turret_q: Query<(&Transform, &OwnedBy), (With<TankTurret>, Without<AimMarker>)>,
    barrel_q: Query<
        (&Transform, &TankBarrelState, &OwnedBy),
        (With<TankBarrel>, Without<AimMarker>),
    >,
    muzzle_q: Query<(&Transform, &OwnedBy), (With<TankMuzzle>, Without<AimMarker>)>,
    rapier_context: ReadRapierContext,
    settings: Res<AimSettings>,
    run_mode: Res<AppRunMode>,
) {
    let Some((mut marker_tf, mut marker_visibility)) = marker_q.iter_mut().next() else {
        return;
    };
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok((player_entity, player_tf, tank_parts, weapon, projectile_profile)) =
        player_q.get(player_entity)
    else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok((turret_tf, turret_owner)) = turret_q.get(tank_parts.turret) else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    if turret_owner.entity != player_entity {
        warn!(
            "TankTurret {:?} is owned by {:?}, expected {:?}",
            tank_parts.turret, turret_owner.entity, player_entity
        );
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok((barrel_tf, barrel_state, barrel_owner)) = barrel_q.get(tank_parts.barrel) else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    if barrel_owner.entity != player_entity {
        warn!(
            "TankBarrel {:?} is owned by {:?}, expected {:?}",
            tank_parts.barrel, barrel_owner.entity, player_entity
        );
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let Ok((muzzle_tf, muzzle_owner)) = muzzle_q.get(tank_parts.muzzle) else {
        *marker_visibility = Visibility::Hidden;
        return;
    };
    if muzzle_owner.entity != player_entity {
        warn!(
            "TankMuzzle {:?} is owned by {:?}, expected {:?}",
            tank_parts.muzzle, muzzle_owner.entity, player_entity
        );
        *marker_visibility = Visibility::Hidden;
        return;
    };
    let artillery_active = barrel_state.artillery_mode_active;
    let Ok(rapier_context) = rapier_context.single() else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    let Some((ray_origin, ray_dir)) =
        muzzle_ray_from_local_hierarchy(player_tf, turret_tf, barrel_tf, muzzle_tf)
    else {
        *marker_visibility = Visibility::Hidden;
        return;
    };

    let mut filter = QueryFilter::new()
        .exclude_collider(player_entity)
        .exclude_rigid_body(player_entity);
    if !matches!(run_mode.0, RunMode::Client) {
        filter = filter.exclude_sensors();
    }
    let projectile_params = build_projectile_spawn_params(
        weapon,
        projectile_profile.copied().unwrap_or_default(),
        artillery_active,
        &settings,
    );
    let impact = predict_impact_from_projectile_params(
        &rapier_context,
        ray_origin,
        ray_dir,
        projectile_params,
        &settings,
        filter,
    );

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
    player_q: Query<(Entity, &TankParts), With<Player>>,
    barrel_q: Query<(&TankBarrelState, &OwnedBy), With<TankBarrel>>,
    settings: Res<AimSettings>,
    mut vignette_q: Query<(&mut BorderColor, &mut BackgroundColor), With<ArtilleryVignette>>,
) {
    let Some((mut border_color, mut bg_color)) = vignette_q.iter_mut().next() else {
        return;
    };

    let artillery_active = resolve_local_player_entity(&local_player_ctx, &local_player_q)
        .and_then(|player_entity| {
            let Ok((player_entity, tank_parts)) = player_q.get(player_entity) else {
                return None;
            };
            let Ok((barrel_state, owned_by)) = barrel_q.get(tank_parts.barrel) else {
                return None;
            };
            if owned_by.entity != player_entity {
                warn!(
                    "TankBarrel {:?} is owned by {:?}, expected {:?}",
                    tank_parts.barrel, owned_by.entity, player_entity
                );
                return None;
            }
            Some(barrel_state.artillery_mode_active)
        })
        .unwrap_or(false);
    let alpha = if artillery_active {
        settings.vignette_alpha
    } else {
        0.0
    };

    *border_color = BorderColor::all(Color::srgba(0.02, 0.03, 0.04, alpha));
    bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha * 0.10);
}

fn predict_impact_from_projectile_params(
    rapier_context: &RapierContext<'_>,
    ray_origin: Vec3,
    ray_dir: Vec3,
    projectile: ProjectileSpawnParams,
    settings: &AimSettings,
    filter: QueryFilter<'_>,
) -> Option<crate::utils::ballistics::BallisticImpact> {
    if projectile.params.gravity <= f32::EPSILON {
        return cast_hitscan_impact(
            rapier_context,
            ray_origin,
            ray_dir,
            projectile.params.max_distance.max(0.0),
            projectile.params.collision_radius.max(0.0),
            filter,
        );
    }

    let step_secs = settings.artillery_step_secs.max(0.005);
    let max_steps_by_lifetime = (projectile.params.max_lifetime_secs / step_secs).ceil() as usize;
    let max_steps = max_steps_by_lifetime
        .max(1)
        .min(settings.artillery_max_steps.max(1));

    predict_ballistic_impact(
        rapier_context,
        ray_origin,
        ray_dir,
        BallisticParams {
            initial_speed: projectile.initial_speed,
            gravity: projectile.params.gravity,
            step_secs,
            max_steps,
            max_distance: projectile.params.max_distance.max(0.0),
            collision_radius: projectile.params.collision_radius.max(0.0),
            downcast_distance: 0.0,
            min_safe_distance: 0.0,
        },
        filter,
    )
}
