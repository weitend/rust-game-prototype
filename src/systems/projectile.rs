use bevy::prelude::*;
use bevy_rapier3d::{
    parry::shape::Ball,
    prelude::{QueryFilter, ReadRapierContext},
};

use crate::{
    components::{
        combat::Health,
        owner::OwnedBy,
        projectile::{Projectile, ProjectileImpactMode},
    },
    systems::impact::ImpactEvent,
    utils::{ballistics::cast_linear_impact, damage_target::resolve_damage_target},
};

pub fn projectile_step_system(
    mut commands: Commands,
    mut impact_events: MessageWriter<ImpactEvent>,
    rapier_context: ReadRapierContext,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
    damageable_targets: Query<(), With<Health>>,
    owned_targets: Query<&OwnedBy>,
    global_tf_q: Query<&GlobalTransform>,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    let dt = time.delta_secs();
    if dt <= f32::EPSILON {
        return;
    }

    for (entity, mut tf, mut projectile) in &mut projectiles {
        if projectile.state.lived_secs >= projectile.params.max_lifetime_secs {
            commands.entity(entity).despawn();
            continue;
        }

        let start = tf.translation;
        let gravity = projectile.params.gravity.max(0.0);
        let accel = Vec3::new(0.0, -gravity, 0.0);
        let next = start + projectile.state.velocity * dt + 0.5 * accel * dt * dt;
        let segment = next - start;
        let segment_len = segment.length();

        if segment_len > f32::EPSILON {
            let dir = segment / segment_len;
            let mut filter = QueryFilter::new().exclude_sensors();
            if let Some(source) = projectile.source {
                filter = filter.exclude_collider(source).exclude_rigid_body(source);
            }

            if let Some(impact) = cast_linear_impact(
                &rapier_context,
                start,
                dir,
                segment_len,
                projectile.params.collision_radius.max(0.0),
                filter,
            ) {
                match projectile.params.impact_mode {
                    ProjectileImpactMode::Direct => {
                        impact_events.write(ImpactEvent {
                            source: projectile.source,
                            target: impact.target,
                            point: impact.point,
                            normal: impact.normal,
                            damage: projectile.params.damage,
                        });
                    }
                    ProjectileImpactMode::Explosion { radius } => {
                        let direct_damage_target = resolve_damage_target(
                            impact.target,
                            &damageable_targets,
                            &owned_targets,
                        );
                        let impact_damage = if direct_damage_target.is_some() {
                            0.0
                        } else {
                            projectile.params.damage
                        };
                        impact_events.write(ImpactEvent {
                            source: projectile.source,
                            target: impact.target,
                            point: impact.point,
                            normal: impact.normal,
                            damage: impact_damage,
                        });

                        let blast_radius = radius.max(projectile.params.collision_radius).max(0.0);
                        if blast_radius > f32::EPSILON {
                            let mut blast_filter = QueryFilter::new().exclude_sensors();
                            if let Some(source) = projectile.source {
                                blast_filter =
                                    blast_filter.exclude_collider(source).exclude_rigid_body(source);
                            }

                            let mut min_distance_by_target = Vec::<(Entity, f32)>::new();
                            rapier_context.intersect_shape(
                                impact.point,
                                Quat::IDENTITY,
                                &Ball::new(blast_radius),
                                blast_filter,
                                |collider_entity| {
                                    let Some(damage_target) = resolve_damage_target(
                                        collider_entity,
                                        &damageable_targets,
                                        &owned_targets,
                                    ) else {
                                        return true;
                                    };

                                    let distance = distance_to_explosion(
                                        impact.point,
                                        collider_entity,
                                        damage_target,
                                        &global_tf_q,
                                        blast_radius,
                                    );
                                    upsert_min_distance(
                                        &mut min_distance_by_target,
                                        damage_target,
                                        distance,
                                    );
                                    true
                                },
                            );

                            if let Some(direct_target) = direct_damage_target {
                                upsert_min_distance(&mut min_distance_by_target, direct_target, 0.0);
                            }

                            min_distance_by_target
                                .sort_by(|(left, _), (right, _)| left.index().cmp(&right.index()));
                            for (damage_target, distance) in min_distance_by_target {
                                let falloff = (1.0 - (distance / blast_radius)).clamp(0.0, 1.0);
                                let damage = projectile.params.damage * falloff;
                                if damage <= f32::EPSILON {
                                    continue;
                                }

                                impact_events.write(ImpactEvent {
                                    source: projectile.source,
                                    target: damage_target,
                                    point: impact.point,
                                    normal: outward_normal(
                                        impact.point,
                                        damage_target,
                                        &global_tf_q,
                                    ),
                                    damage,
                                });
                            }
                        }
                    }
                }
                commands.entity(entity).despawn();
                continue;
            }

            projectile.state.traveled_distance += segment_len;
            if projectile.state.traveled_distance >= projectile.params.max_distance.max(0.0) {
                commands.entity(entity).despawn();
                continue;
            }
        }

        tf.translation = next;
        projectile.state.velocity += accel * dt;
        projectile.state.lived_secs += dt;
        if projectile.state.lived_secs >= projectile.params.max_lifetime_secs {
            commands.entity(entity).despawn();
        }
    }
}

fn upsert_min_distance(entries: &mut Vec<(Entity, f32)>, target: Entity, distance: f32) {
    if let Some((_, existing)) = entries.iter_mut().find(|(entity, _)| *entity == target) {
        *existing = existing.min(distance);
    } else {
        entries.push((target, distance));
    }
}

fn distance_to_explosion(
    center: Vec3,
    collider_entity: Entity,
    damage_target: Entity,
    global_tf_q: &Query<&GlobalTransform>,
    fallback: f32,
) -> f32 {
    if let Ok(tf) = global_tf_q.get(collider_entity) {
        return tf.translation().distance(center);
    }
    if let Ok(tf) = global_tf_q.get(damage_target) {
        return tf.translation().distance(center);
    }
    fallback
}

fn outward_normal(center: Vec3, target: Entity, global_tf_q: &Query<&GlobalTransform>) -> Vec3 {
    let dir = global_tf_q
        .get(target)
        .ok()
        .map(|tf| tf.translation() - center)
        .unwrap_or(Vec3::Y);
    let normal = dir.normalize_or_zero();
    if normal == Vec3::ZERO {
        Vec3::Y
    } else {
        normal
    }
}
