use bevy::{ecs::entity::Entity, prelude::Vec3};
use bevy_rapier3d::prelude::{QueryFilter, RapierContext};

#[derive(Clone, Copy, Debug)]
pub struct BallisticParams {
    pub initial_speed: f32,
    pub gravity: f32,
    pub step_secs: f32,
    pub max_steps: usize,
    pub max_distance: f32,
    pub downcast_distance: f32,
    pub min_safe_distance: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BallisticImpact {
    pub target: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub travel_distance: f32,
}

pub fn predict_ballistic_impact(
    rapier_context: &RapierContext<'_>,
    origin: Vec3,
    direction: Vec3,
    params: BallisticParams,
    filter: QueryFilter<'_>,
) -> Option<BallisticImpact> {
    if params.initial_speed <= f32::EPSILON
        || params.step_secs <= f32::EPSILON
        || params.max_steps == 0
        || params.max_distance <= f32::EPSILON
    {
        return None;
    }

    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let gravity = Vec3::new(0.0, -params.gravity.max(0.0), 0.0);
    let step = params.step_secs;
    let mut pos = origin;
    let mut vel = dir * params.initial_speed;
    let mut traveled = 0.0f32;

    for _ in 0..params.max_steps {
        let next = pos + vel * step + 0.5 * gravity * step * step;
        let segment = next - pos;
        let segment_len = segment.length();
        if segment_len <= f32::EPSILON {
            break;
        }

        let seg_dir = segment / segment_len;
        if let Some((target, hit)) =
            rapier_context.cast_ray_and_get_normal(pos, seg_dir, segment_len, true, filter)
        {
            let hit_distance = traveled + hit.time_of_impact.max(0.0);
            if hit_distance >= params.min_safe_distance {
                let normal = hit.normal.normalize_or_zero();
                return Some(BallisticImpact {
                    target,
                    point: hit.point,
                    normal: if normal == Vec3::ZERO {
                        Vec3::Y
                    } else {
                        normal
                    },
                    travel_distance: hit_distance,
                });
            }
        }

        traveled += segment_len;
        if traveled >= params.max_distance {
            break;
        }

        pos = next;
        vel += gravity * step;
    }

    if params.downcast_distance > f32::EPSILON {
        if let Some((target, hit)) = rapier_context.cast_ray_and_get_normal(
            pos,
            -Vec3::Y,
            params.downcast_distance,
            true,
            filter,
        ) {
            let hit_distance = traveled + hit.time_of_impact.max(0.0);
            if hit_distance >= params.min_safe_distance {
                let normal = hit.normal.normalize_or_zero();
                return Some(BallisticImpact {
                    target,
                    point: hit.point,
                    normal: if normal == Vec3::ZERO {
                        Vec3::Y
                    } else {
                        normal
                    },
                    travel_distance: hit_distance,
                });
            }
        }
    }

    None
}

pub fn cast_hitscan_impact(
    rapier_context: &RapierContext<'_>,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    filter: QueryFilter<'_>,
) -> Option<BallisticImpact> {
    if max_distance <= f32::EPSILON {
        return None;
    }

    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    rapier_context
        .cast_ray_and_get_normal(origin, dir, max_distance, true, filter)
        .map(|(target, hit)| {
            let normal = hit.normal.normalize_or_zero();
            BallisticImpact {
                target,
                point: hit.point,
                normal: if normal == Vec3::ZERO {
                    Vec3::Y
                } else {
                    normal
                },
                travel_distance: hit.time_of_impact.max(0.0),
            }
        })
}
