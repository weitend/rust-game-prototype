use bevy::{math::Affine3A, prelude::*};

use crate::{
    components::destructible_surface::DestructibleSurface,
    resources::impact_assets::ImpactAssets,
    utils::impact_math::{
        marks_per_impact, normalized_or_up, sample_web_mark, web_radius_for_damage,
    },
};

#[derive(Clone, Copy)]
pub(super) struct ImpactSample {
    pub(super) world_point: Vec3,
    pub(super) size_scale: f32,
    pub(super) depth_scale: f32,
}

pub(super) fn collect_new_impact_samples(
    impact_point: Vec3,
    impact_normal: Vec3,
    damage: f32,
    impact_assets: &ImpactAssets,
    surface: &mut DestructibleSurface,
    world_to_local: &Affine3A,
    max_samples: usize,
) -> Vec<ImpactSample> {
    if max_samples == 0 {
        return Vec::new();
    }

    let normal = normalized_or_up(impact_normal);
    let tangent = normal.any_orthonormal_vector();
    let bitangent = normal.cross(tangent).normalize_or_zero();

    let mark_count = marks_per_impact(damage, impact_assets);
    let web_radius = web_radius_for_damage(damage, impact_assets);
    let mut samples = Vec::with_capacity(mark_count);

    for mark_index in 0..mark_count {
        let sample = sample_web_mark(mark_index, mark_count, web_radius, impact_point);
        let world_point = impact_point + tangent * sample.offset.x + bitangent * sample.offset.y;
        let local_point = world_to_local.transform_point3(world_point);

        if !surface.try_mark(local_point) {
            continue;
        }

        samples.push(ImpactSample {
            world_point,
            size_scale: sample.size_scale,
            depth_scale: sample.depth_scale,
        });

        if samples.len() >= max_samples {
            break;
        }
    }

    samples
}
