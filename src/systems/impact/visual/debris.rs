use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::debris_chip::DebrisChip,
    resources::impact_assets::ImpactAssets,
    utils::{
        collision_groups::debris_collision_groups,
        impact_math::{hash01, normalized_or_up},
    },
};

pub fn debris_chip_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut chips: Query<(Entity, &mut DebrisChip)>,
) {
    for (entity, mut chip) in &mut chips {
        chip.timer.tick(time.delta());
        if chip.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn spawn_debris_chip(
    commands: &mut Commands,
    impact_assets: &ImpactAssets,
    material: Handle<StandardMaterial>,
    origin: Vec3,
    impact_normal: Vec3,
    seed_offset: f32,
) {
    let normal = normalized_or_up(impact_normal);
    let tangent = normal.any_orthonormal_vector();
    let bitangent = normal.cross(tangent).normalize_or_zero();

    let seed = origin + Vec3::new(0.31, 0.73, seed_offset * 0.19);
    let angle = hash01(seed + Vec3::new(0.11, 0.47, 0.89)) * std::f32::consts::TAU;
    let radial = hash01(seed + Vec3::new(0.37, 0.13, 0.71));
    let lateral = (tangent * angle.cos() + bitangent * angle.sin()).normalize_or_zero();
    let launch = normalized_or_up(normal + lateral * (0.45 + radial * 0.8) + Vec3::Y * 0.28);
    let scale = 0.65 + 0.55 * radial;
    let size = impact_assets.chip_size * scale;
    let half = 0.5 * size;

    commands.spawn((
        Mesh3d(impact_assets.chip_mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(origin + normal * (impact_assets.radius * 0.35))
            .with_scale(Vec3::splat(scale)),
        RigidBody::Dynamic,
        Collider::cuboid(half, half, half),
        debris_collision_groups(),
        Ccd::enabled(),
        Velocity {
            linvel: launch * impact_assets.chip_speed,
            angvel: Vec3::new(6.0 * scale, -4.2 * scale, 3.5 * scale),
        },
        Damping {
            linear_damping: 1.8,
            angular_damping: 3.0,
        },
        DebrisChip {
            timer: Timer::from_seconds(impact_assets.chip_lifetime_secs, TimerMode::Once),
        },
    ));
}

