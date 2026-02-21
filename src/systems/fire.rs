use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        fire_control::FireControl,
        player::Player,
        shoot_origin::ShootOrigin,
        shot_tracer::{ShotTracer, ShotTracerLifetime},
        weapon::HitscanWeapon,
    },
    resources::tracer_assets::TracerAssets,
    systems::impact::ImpactEvent,
};

pub fn fire_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut impact_events: MessageWriter<ImpactEvent>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &ShootOrigin,
            &mut FireControl,
            &HitscanWeapon,
        ),
        With<Player>,
    >,
    rapier_context: ReadRapierContext,
    tracer_assets: Res<TracerAssets>,
    time: Res<Time>,
) {
    let Ok((player_entity, player_tf, shoot_origin, mut fire_control, weapon)) = query.single_mut()
    else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        fire_control.cooldown.reset();
        return;
    }

    fire_control.cooldown.tick(time.delta());

    if !mouse.just_pressed(MouseButton::Left) && !fire_control.cooldown.just_finished() {
        return;
    }

    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    let ray_origin = player_tf.translation + player_tf.rotation * shoot_origin.muzzle_offset;
    let ray_dir = (player_tf.rotation * -Vec3::Z).normalize_or_zero();

    if ray_dir == Vec3::ZERO {
        return;
    }

    let filter = QueryFilter::new()
        .exclude_collider(player_entity)
        .exclude_rigid_body(player_entity)
        .exclude_sensors();

    let ray_result =
        rapier_context.cast_ray_and_get_normal(ray_origin, ray_dir, weapon.range, true, filter);

    let travel_distance = ray_result
        .map(|(_, hit)| hit.time_of_impact)
        .unwrap_or(weapon.range)
        .max(0.0);
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

    let Some((hit_entity, hit)) = ray_result else {
        return;
    };

    impact_events.write(ImpactEvent {
        source: Some(player_entity),
        target: hit_entity,
        point: hit.point,
        normal: hit.normal,
        damage: weapon.damage,
    });
}
