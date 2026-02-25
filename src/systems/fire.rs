use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        fire_control::FireControl,
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player},
        shot_tracer::{ShotTracer, ShotTracerLifetime},
        tank::{TankBarrel, TankBarrelState, TankMuzzle, TankParts},
        weapon::HitscanWeapon,
    },
    resources::{
        aim_settings::AimSettings,
        run_mode::{AppRunMode, RunMode},
        tracer_assets::TracerAssets,
    },
    systems::impact::ImpactEvent,
    utils::ballistics::{cast_hitscan_impact, predict_ballistic_impact},
    utils::muzzle::muzzle_ray,
};

pub fn fire_system(
    mut commands: Commands,
    mut impact_events: MessageWriter<ImpactEvent>,
    run_mode: Res<AppRunMode>,
    mut player_q: Query<
        (
            Entity,
            &TankParts,
            &mut FireControl,
            &HitscanWeapon,
            &PlayerIntent,
            Option<&LocalPlayer>,
        ),
        With<Player>,
    >,
    muzzle_q: Query<(&GlobalTransform, &OwnedBy), With<TankMuzzle>>,
    barrel_q: Query<(&TankBarrelState, &OwnedBy), With<TankBarrel>>,
    rapier_context: ReadRapierContext,
    tracer_assets: Option<Res<TracerAssets>>,
    aim_settings: Res<AimSettings>,
    time: Res<Time>,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    for (player_entity, tank_parts, mut fire_control, weapon, intent, local_marker) in &mut player_q
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

        let Ok((muzzle_tf, owned_by)) = muzzle_q.get(tank_parts.muzzle) else {
            continue;
        };
        if owned_by.entity != player_entity {
            warn!(
                "TankMuzzle {:?} is owned by {:?}, expected {:?}",
                tank_parts.muzzle, owned_by.entity, player_entity
            );
            continue;
        };
        let Ok((barrel_state, barrel_owner)) = barrel_q.get(tank_parts.barrel) else {
            continue;
        };
        if barrel_owner.entity != player_entity {
            warn!(
                "TankBarrel {:?} is owned by {:?}, expected {:?}",
                tank_parts.barrel, barrel_owner.entity, player_entity
            );
            continue;
        };
        let artillery_active = barrel_state.artillery_mode_active;

        let Some((ray_origin, ray_dir)) = muzzle_ray(muzzle_tf) else {
            continue;
        };

        let filter = QueryFilter::new()
            .exclude_collider(player_entity)
            .exclude_rigid_body(player_entity)
            .exclude_sensors();

        let (travel_distance, impact) = if artillery_active {
            let ballistic = predict_ballistic_impact(
                &rapier_context,
                ray_origin,
                ray_dir,
                aim_settings.artillery_ballistic_params(weapon.range),
                filter,
            );
            let distance = ballistic
                .map(|hit| hit.travel_distance)
                .unwrap_or(aim_settings.effective_range(weapon.range))
                .max(0.0);
            (distance, ballistic)
        } else {
            let hitscan =
                cast_hitscan_impact(&rapier_context, ray_origin, ray_dir, weapon.range, filter);
            let distance = hitscan
                .map(|hit| hit.travel_distance)
                .unwrap_or(weapon.range)
                .max(0.0);
            (distance, hitscan)
        };

        if let Some(tracer_assets) = tracer_assets.as_ref() {
            let tracer_speed = tracer_assets.speed.max(1.0);
            let tracer_lifetime = (travel_distance / tracer_speed).max(0.01);

            commands.spawn((
                Mesh3d(tracer_assets.mesh.clone()),
                MeshMaterial3d(tracer_assets.material.clone()),
                Transform::from_translation(ray_origin),
                ShotTracer {
                    velocity: ray_dir * tracer_speed,
                },
                ShotTracerLifetime {
                    timer: Timer::from_seconds(tracer_lifetime, TimerMode::Once),
                },
            ));
        }

        let Some(impact) = impact else {
            continue;
        };

        impact_events.write(ImpactEvent {
            source: Some(player_entity),
            target: impact.target,
            point: impact.point,
            normal: impact.normal,
            damage: weapon.damage,
        });
        if !matches!(run_mode.0, RunMode::Client) && intent.fire_just_pressed {
            eprintln!(
                "[fire-auth] source={:?} target={:?} damage={:.1}",
                player_entity, impact.target, weapon.damage
            );
        }
    }
}
