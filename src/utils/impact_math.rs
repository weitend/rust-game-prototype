use bevy::prelude::*;

use crate::resources::impact_assets::ImpactAssets;

#[derive(Clone, Copy)]
pub struct WebMarkSample {
    pub offset: Vec2,
    pub size_scale: f32,
    pub depth_scale: f32,
}

pub fn sample_web_mark(
    mark_index: usize,
    mark_count: usize,
    web_radius: f32,
    seed_point: Vec3,
) -> WebMarkSample {
    if mark_index == 0 || mark_count <= 1 {
        return WebMarkSample {
            offset: Vec2::ZERO,
            size_scale: 1.25,
            depth_scale: 1.15,
        };
    }

    let ring_count = mark_count.saturating_sub(1).max(1);
    let ring_idx = mark_index.saturating_sub(1);
    let ring_t = ring_idx as f32 / ring_count as f32;
    let arm_count = (4 + mark_count / 2).clamp(4, 8);
    let arm_idx = ring_idx % arm_count;
    let arm_phase = arm_idx as f32 / arm_count as f32 * std::f32::consts::TAU;

    let seed_base = seed_point + Vec3::new(mark_index as f32 * 0.31, 0.73, 0.19);
    let jitter_angle = (hash01(seed_base + Vec3::new(0.11, 0.47, 0.89)) - 0.5) * 0.56;
    let jitter_radius = 0.78 + 0.36 * hash01(seed_base + Vec3::new(0.37, 0.13, 0.71));
    let radius = web_radius * (0.25 + 0.75 * ring_t.sqrt()) * jitter_radius;
    let angle = arm_phase + jitter_angle;
    let offset = Vec2::new(angle.cos(), angle.sin()) * radius;

    WebMarkSample {
        offset,
        size_scale: (1.05 - 0.5 * ring_t).clamp(0.42, 1.08),
        depth_scale: (1.1 - 0.6 * ring_t).clamp(0.4, 1.1),
    }
}

pub fn marks_per_impact(damage: f32, impact_assets: &ImpactAssets) -> usize {
    let min_marks = impact_assets.min_marks_per_impact.max(1);
    let max_marks = impact_assets.max_marks_per_impact.max(min_marks);
    let factor = if impact_assets.damage_for_max_web <= f32::EPSILON {
        1.0
    } else {
        (damage / impact_assets.damage_for_max_web).clamp(0.0, 1.0)
    };
    let target = min_marks as f32 + (max_marks - min_marks) as f32 * factor;
    target.round() as usize
}

pub fn chips_per_impact(damage: f32, impact_assets: &ImpactAssets) -> usize {
    let min_chips = impact_assets.min_chips_per_impact.max(1);
    let max_chips = impact_assets.max_chips_per_impact.max(min_chips);
    let factor = if impact_assets.damage_for_max_web <= f32::EPSILON {
        1.0
    } else {
        (damage / impact_assets.damage_for_max_web).clamp(0.0, 1.0)
    };
    let target = min_chips as f32 + (max_chips - min_chips) as f32 * factor;
    target.round() as usize
}

pub fn web_radius_for_damage(damage: f32, impact_assets: &ImpactAssets) -> f32 {
    let base = impact_assets.base_web_radius.max(0.01);
    let max = impact_assets.max_web_radius.max(base);
    let factor = if impact_assets.damage_for_max_web <= f32::EPSILON {
        1.0
    } else {
        (damage / impact_assets.damage_for_max_web).clamp(0.0, 1.0)
    };
    base + (max - base) * factor
}

pub fn normalized_or_up(v: Vec3) -> Vec3 {
    let n = v.normalize_or_zero();
    if n == Vec3::ZERO {
        Vec3::Y
    } else {
        n
    }
}

pub(crate) fn hash01(v: Vec3) -> f32 {
    let dot = v.dot(Vec3::new(12.9898, 78.233, 45.164));
    (dot.sin() * 43_758.547).fract().abs()
}
