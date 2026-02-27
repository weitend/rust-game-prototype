use bevy::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        owner::OwnedBy,
        shot_tracer::{ExplosionVfx, ShotTracer, ShotTracerLifetime, SmokePuff},
    },
    resources::tracer_assets::TracerAssets,
    systems::impact::ImpactEvent,
    utils::damage_target::resolve_damage_target,
};

pub fn update_shot_tracer_system(
    mut commands: Commands,
    time: Res<Time>,
    tracer_assets: Option<Res<TracerAssets>>,
    mut tracers: Query<(
        Entity,
        &mut ShotTracer,
        &mut Transform,
        &mut ShotTracerLifetime,
    )>,
) {
    let dt = time.delta_secs();
    let delta = time.delta();

    for (entity, mut tracer, mut transform, mut lifetime) in &mut tracers {
        transform.translation += tracer.velocity * dt;
        let forward = tracer.velocity.normalize_or_zero();
        if forward != Vec3::ZERO {
            transform.look_to(forward, Vec3::Y);
        }

        if let Some(tracer_assets) = tracer_assets.as_ref() {
            tracer.smoke_timer.tick(delta);
            if tracer.smoke_timer.just_finished() {
                spawn_trail_smoke(
                    &mut commands,
                    tracer_assets,
                    transform.translation,
                    tracer.velocity,
                );
            }
        }

        lifetime.timer.tick(delta);
        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_smoke_puff_system(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke_q: Query<(Entity, &mut Transform, &mut SmokePuff)>,
) {
    let dt = time.delta_secs();
    let delta = time.delta();

    for (entity, mut tf, mut puff) in &mut smoke_q {
        puff.timer.tick(delta);
        let duration = puff.timer.duration().as_secs_f32().max(0.001);
        let t = (puff.timer.elapsed_secs() / duration).clamp(0.0, 1.0);

        tf.translation += puff.velocity * dt;
        puff.velocity *= (1.0 - 1.8 * dt).clamp(0.0, 1.0);
        puff.velocity.y += 0.55 * dt;
        let scale = puff.start_scale + (puff.end_scale - puff.start_scale) * t;
        tf.scale = Vec3::splat(scale.max(0.001));

        if puff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_hit_explosion_system(
    mut commands: Commands,
    tracer_assets: Option<Res<TracerAssets>>,
    mut impact_events: MessageReader<ImpactEvent>,
    damageable_targets: Query<(), With<Health>>,
    owned_targets: Query<&OwnedBy>,
    teams_q: Query<&Team>,
) {
    let Some(tracer_assets) = tracer_assets.as_ref() else {
        return;
    };

    for impact in impact_events.read() {
        let Some(target) =
            resolve_damage_target(impact.target, &damageable_targets, &owned_targets)
        else {
            continue;
        };
        let Ok(team) = teams_q.get(target) else {
            continue;
        };
        if *team != Team::Enemy {
            continue;
        }

        let normal = {
            let n = impact.normal.normalize_or_zero();
            if n == Vec3::ZERO { Vec3::Y } else { n }
        };

        let end_scale = (0.70 + impact.damage * 0.02).clamp(0.70, 1.90);
        commands.spawn((
            Mesh3d(tracer_assets.explosion_mesh.clone()),
            MeshMaterial3d(tracer_assets.explosion_material.clone()),
            Transform::from_translation(impact.point + normal * 0.10).with_scale(Vec3::splat(0.10)),
            ExplosionVfx {
                timer: Timer::from_seconds(0.20, TimerMode::Once),
                start_scale: 0.10,
                end_scale,
            },
        ));

        let tangent = normal.any_orthonormal_vector();
        let bitangent = normal.cross(tangent).normalize_or_zero();
        for idx in 0..6 {
            let phase = idx as f32 / 6.0;
            let angle = phase * std::f32::consts::TAU;
            let ring = tangent * angle.cos() + bitangent * angle.sin();
            let spread = 0.55 + 0.45 * (idx as f32 * 1.43).sin().abs();
            let start_scale = 0.08 + 0.03 * spread;

            commands.spawn((
                Mesh3d(tracer_assets.smoke_mesh.clone()),
                MeshMaterial3d(tracer_assets.smoke_material.clone()),
                Transform::from_translation(impact.point + normal * 0.08 + ring * 0.05)
                    .with_scale(Vec3::splat(start_scale)),
                SmokePuff {
                    velocity: normal * (1.2 + spread * 1.0) + ring * (0.8 + spread * 0.8),
                    timer: Timer::from_seconds(0.32 + 0.26 * spread, TimerMode::Once),
                    start_scale,
                    end_scale: 0.32 + 0.20 * spread,
                },
            ));
        }
    }
}

pub fn update_explosion_vfx_system(
    mut commands: Commands,
    time: Res<Time>,
    mut explosion_q: Query<(Entity, &mut Transform, &mut ExplosionVfx)>,
) {
    let delta = time.delta();

    for (entity, mut tf, mut explosion) in &mut explosion_q {
        explosion.timer.tick(delta);
        let duration = explosion.timer.duration().as_secs_f32().max(0.001);
        let t = (explosion.timer.elapsed_secs() / duration).clamp(0.0, 1.0);
        let scale = explosion.start_scale + (explosion.end_scale - explosion.start_scale) * t;
        tf.scale = Vec3::splat(scale.max(0.001));

        if explosion.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_trail_smoke(
    commands: &mut Commands,
    tracer_assets: &TracerAssets,
    position: Vec3,
    velocity: Vec3,
) {
    let dir = velocity.normalize_or_zero();
    let lateral = Vec3::new(
        (position.x * 12.3).sin() * 0.35,
        0.35,
        (position.z * 8.7).cos() * 0.35,
    );
    let start_scale = 0.055;
    let end_scale = 0.20;

    commands.spawn((
        Mesh3d(tracer_assets.smoke_mesh.clone()),
        MeshMaterial3d(tracer_assets.smoke_material.clone()),
        Transform::from_translation(position - dir * 0.02).with_scale(Vec3::splat(start_scale)),
        SmokePuff {
            velocity: -dir * 0.9 + lateral,
            timer: Timer::from_seconds(0.28, TimerMode::Once),
            start_scale,
            end_scale,
        },
    ));
}
