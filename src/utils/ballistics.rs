use bevy::{
    ecs::entity::Entity,
    prelude::{Quat, Vec3},
};
use bevy_rapier3d::{
    parry::shape::Ball,
    prelude::{QueryFilter, RapierContext, ShapeCastOptions},
};

#[derive(Clone, Copy, Debug)]
pub struct BallisticParams {
    pub initial_speed: f32,
    pub gravity: f32,
    pub step_secs: f32,
    pub max_steps: usize,
    pub max_distance: f32,
    pub collision_radius: f32,
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
        if let Some(hit) = cast_linear_impact(
            rapier_context,
            pos,
            seg_dir,
            segment_len,
            params.collision_radius,
            filter,
        ) {
            let hit_distance = traveled + hit.travel_distance.max(0.0);
            if hit_distance >= params.min_safe_distance {
                return Some(BallisticImpact {
                    target: hit.target,
                    point: hit.point,
                    normal: hit.normal,
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
        if let Some(hit) = cast_linear_impact(
            rapier_context,
            pos,
            -Vec3::Y,
            params.downcast_distance,
            params.collision_radius,
            filter,
        ) {
            let hit_distance = traveled + hit.travel_distance.max(0.0);
            if hit_distance >= params.min_safe_distance {
                return Some(BallisticImpact {
                    target: hit.target,
                    point: hit.point,
                    normal: hit.normal,
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
    collision_radius: f32,
    filter: QueryFilter<'_>,
) -> Option<BallisticImpact> {
    cast_linear_impact(
        rapier_context,
        origin,
        direction,
        max_distance,
        collision_radius,
        filter,
    )
}

pub fn cast_linear_impact(
    rapier_context: &RapierContext<'_>,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    collision_radius: f32,
    filter: QueryFilter<'_>,
) -> Option<BallisticImpact> {
    if max_distance <= f32::EPSILON {
        return None;
    }

    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let radius = collision_radius.max(0.0);
    if radius <= f32::EPSILON {
        return rapier_context
            .cast_ray_and_get_normal(origin, dir, max_distance, true, filter)
            .map(|(target, hit)| BallisticImpact {
                target,
                point: hit.point,
                normal: normalize_or_up(hit.normal),
                travel_distance: hit.time_of_impact.max(0.0),
            });
    }

    let shape = Ball::new(radius);
    let options = ShapeCastOptions::with_max_time_of_impact(max_distance);
    rapier_context
        .cast_shape(origin, Quat::IDENTITY, dir, &shape, options, filter)
        .map(|(target, hit)| {
            let fallback_point = origin + dir * hit.time_of_impact.max(0.0);
            let (point, normal) = hit
                .details
                .map(|details| (details.witness1, details.normal1))
                .unwrap_or((fallback_point, Vec3::Y));
            BallisticImpact {
                target,
                point,
                normal: normalize_or_up(normal),
                travel_distance: hit.time_of_impact.max(0.0),
            }
        })
}

fn normalize_or_up(v: Vec3) -> Vec3 {
    let normalized = v.normalize_or_zero();
    if normalized == Vec3::ZERO {
        Vec3::Y
    } else {
        normalized
    }
}
