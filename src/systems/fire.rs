use bevy::prelude::*;

use crate::{
    components::{
        fire_control::FireControl,
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        projectile::Projectile,
        shot_tracer::{ShotTracer, ShotTracerLifetime},
        tank::{TankBarrel, TankBarrelState, TankMuzzle, TankParts, TankTurret},
        weapon::{HitscanWeapon, ProjectileWeaponProfile},
    },
    resources::{
        aim_settings::AimSettings,
        run_mode::{AppRunMode, RunMode},
        tracer_assets::TracerAssets,
    },
    utils::{
        muzzle::muzzle_ray_from_local_hierarchy,
        weapon_ballistics::build_projectile_spawn_params,
    },
};

pub fn fire_system(
    mut commands: Commands,
    run_mode: Res<AppRunMode>,
    mut player_q: Query<
        (
            Entity,
            &Transform,
            &TankParts,
            &mut FireControl,
            &HitscanWeapon,
            Option<&ProjectileWeaponProfile>,
            &PlayerIntent,
            Option<&LocalPlayer>,
        ),
        With<Player>,
    >,
    turret_q: Query<(&Transform, &OwnedBy), With<TankTurret>>,
    barrel_q: Query<(&Transform, &TankBarrelState, &OwnedBy), With<TankBarrel>>,
    muzzle_q: Query<(&Transform, &OwnedBy), With<TankMuzzle>>,
    tracer_assets: Option<Res<TracerAssets>>,
    aim_settings: Res<AimSettings>,
    time: Res<Time>,
) {
    for (player_entity, player_tf, tank_parts, mut fire_control, weapon, projectile_profile, intent, local_marker) in &mut player_q
    {
        let simulate_fire = match run_mode.0 {
            RunMode::Client => local_marker.is_some(),
            RunMode::Server | RunMode::Host => true,
        };
        if !simulate_fire {
            continue;
        }

        if !intent.fire_pressed {
            fire_control.cooldown.reset();
            continue;
        }

        fire_control.cooldown.tick(time.delta());

        if !intent.fire_just_pressed && !fire_control.cooldown.just_finished() {
            continue;
        }

        let Ok((turret_tf, turret_owner)) = turret_q.get(tank_parts.turret) else {
            continue;
        };
        if turret_owner.entity != player_entity {
            warn!(
                "TankTurret {:?} is owned by {:?}, expected {:?}",
                tank_parts.turret, turret_owner.entity, player_entity
            );
            continue;
        }
        let Ok((barrel_tf, barrel_state, barrel_owner)) = barrel_q.get(tank_parts.barrel) else {
            continue;
        };
        if barrel_owner.entity != player_entity {
            warn!(
                "TankBarrel {:?} is owned by {:?}, expected {:?}",
                tank_parts.barrel, barrel_owner.entity, player_entity
            );
            continue;
        };
        let Ok((muzzle_tf, muzzle_owner)) = muzzle_q.get(tank_parts.muzzle) else {
            continue;
        };
        if muzzle_owner.entity != player_entity {
            warn!(
                "TankMuzzle {:?} is owned by {:?}, expected {:?}",
                tank_parts.muzzle, muzzle_owner.entity, player_entity
            );
            continue;
        };
        let artillery_active = barrel_state.artillery_mode_active;

        let Some((ray_origin, ray_dir)) =
            muzzle_ray_from_local_hierarchy(player_tf, turret_tf, barrel_tf, muzzle_tf)
        else {
            continue;
        };

        let projectile_params = build_projectile_spawn_params(
            weapon,
            projectile_profile.copied().unwrap_or_default(),
            artillery_active,
            &aim_settings,
        );
        let projectile_speed = projectile_params.initial_speed.max(1.0);
        let tracer_travel_distance = projectile_params.params.max_distance.max(0.0);

        if let Some(tracer_assets) = tracer_assets.as_ref() {
            let tracer_lifetime = (tracer_travel_distance / projectile_speed).max(0.01);

            commands.spawn((
                Mesh3d(tracer_assets.mesh.clone()),
                MeshMaterial3d(tracer_assets.material.clone()),
                Transform::from_translation(ray_origin),
                ShotTracer {
                    velocity: ray_dir * projectile_speed,
                },
                ShotTracerLifetime {
                    timer: Timer::from_seconds(tracer_lifetime, TimerMode::Once),
                },
            ));
        }

        if matches!(run_mode.0, RunMode::Server | RunMode::Host) {
            commands.spawn((
                Transform::from_translation(ray_origin),
                Projectile::with_params(
                    Some(player_entity),
                    projectile_params.params,
                    ray_dir * projectile_speed,
                ),
            ));
        }
    }
}
