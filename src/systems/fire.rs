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
        aim_settings::AimSettings, local_player::LocalPlayerContext, tracer_assets::TracerAssets,
    },
    systems::impact::ImpactEvent,
    utils::{
        ballistics::predict_ballistic_impact, local_player::resolve_local_player_entity,
        muzzle::muzzle_ray,
    },
};

pub fn fire_system(
    mut commands: Commands,
    mut impact_events: MessageWriter<ImpactEvent>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut player_q: Query<
        (
            Entity,
            &TankParts,
            &mut FireControl,
            &HitscanWeapon,
            &PlayerIntent,
        ),
        With<Player>,
    >,
    muzzle_q: Query<(&GlobalTransform, &OwnedBy), With<TankMuzzle>>,
    barrel_q: Query<(&TankBarrelState, &OwnedBy), With<TankBarrel>>,
    rapier_context: ReadRapierContext,
    tracer_assets: Res<TracerAssets>,
    aim_settings: Res<AimSettings>,
    time: Res<Time>,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((player_entity, tank_parts, mut fire_control, weapon, intent)) =
        player_q.get_mut(player_entity)
    else {
        return;
    };

    if !intent.fire_pressed {
        fire_control.cooldown.reset();
        return;
    }

    fire_control.cooldown.tick(time.delta());

    if !intent.fire_just_pressed && !fire_control.cooldown.just_finished() {
        return;
    }

    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };
    let Ok((muzzle_tf, owned_by)) = muzzle_q.get(tank_parts.muzzle) else {
        return;
    };
    if owned_by.entity != player_entity {
        warn!(
            "TankMuzzle {:?} is owned by {:?}, expected {:?}",
            tank_parts.muzzle, owned_by.entity, player_entity
        );
        return;
    };
    let Ok((barrel_state, barrel_owner)) = barrel_q.get(tank_parts.barrel) else {
        return;
    };
    if barrel_owner.entity != player_entity {
        warn!(
            "TankBarrel {:?} is owned by {:?}, expected {:?}",
            tank_parts.barrel, barrel_owner.entity, player_entity
        );
        return;
    };
    let artillery_active = barrel_state.artillery_mode_active;

    let Some((ray_origin, ray_dir)) = muzzle_ray(muzzle_tf) else {
        return;
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
        let ray_result =
            rapier_context.cast_ray_and_get_normal(ray_origin, ray_dir, weapon.range, true, filter);
        let distance = ray_result
            .map(|(_, hit)| hit.time_of_impact)
            .unwrap_or(weapon.range)
            .max(0.0);
        let hit = ray_result.map(|(target, hit)| crate::utils::ballistics::BallisticImpact {
            target,
            point: hit.point,
            normal: hit.normal,
            travel_distance: hit.time_of_impact.max(0.0),
        });
        (distance, hit)
    };

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

    let Some(impact) = impact else {
        return;
    };

    impact_events.write(ImpactEvent {
        source: Some(player_entity),
        target: impact.target,
        point: impact.point,
        normal: impact.normal,
        damage: weapon.damage,
    });
}
