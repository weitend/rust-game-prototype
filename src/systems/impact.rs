use bevy::math::Affine3A;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::Health, debris_chip::DebrisChip, destructible_mesh::DestructibleMesh,
        destructible_surface::DestructibleSurface, obstacle::Obstacle,
    },
    resources::impact_assets::ImpactAssets,
    systems::combat::DamageEvent,
    utils::collision_groups::debris_collision_groups,
};

type ObstacleImpactQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GlobalTransform,
        &'static mut DestructibleSurface,
        &'static Mesh3d,
        Option<&'static MeshMaterial3d<StandardMaterial>>,
        Option<&'static DestructibleMesh>,
    ),
    With<Obstacle>,
>;

#[derive(Default)]
struct ImpactFrameBudget {
    dents_remaining: usize,
    chips_remaining: usize,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct ImpactEvent {
    pub source: Option<Entity>,
    pub target: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub damage: f32,
}

pub fn process_impact_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut impact_events: MessageReader<ImpactEvent>,
    mut damage_events: MessageWriter<DamageEvent>,
    mut obstacles: ObstacleImpactQuery<'_, '_>,
    damageable_targets: Query<(), With<Health>>,
    impact_assets: Res<ImpactAssets>,
) {
    let mut frame_budget = ImpactFrameBudget {
        dents_remaining: impact_assets.max_marks_per_frame,
        chips_remaining: impact_assets.max_chips_per_frame,
    };

    for impact in impact_events.read() {
        route_damage(impact, &damageable_targets, &mut damage_events);
        process_obstacle_impact(
            &mut commands,
            &mut meshes,
            &mut obstacles,
            impact,
            &impact_assets,
            &mut frame_budget,
        );

        if frame_budget.dents_remaining == 0 && frame_budget.chips_remaining == 0 {
            break;
        }
    }
}

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

#[derive(Clone, Copy)]
struct ImpactSample {
    world_point: Vec3,
    size_scale: f32,
    depth_scale: f32,
}

fn route_damage(
    impact: &ImpactEvent,
    damageable_targets: &Query<(), With<Health>>,
    damage_events: &mut MessageWriter<DamageEvent>,
) {
    if damageable_targets.contains(impact.target) {
        damage_events.write(DamageEvent {
            source: impact.source,
            target: impact.target,
            amount: impact.damage,
        });
    }
}

fn process_obstacle_impact(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    obstacles: &mut ObstacleImpactQuery<'_, '_>,
    impact: &ImpactEvent,
    impact_assets: &ImpactAssets,
    frame_budget: &mut ImpactFrameBudget,
) {
    let Ok((obstacle_tf, mut surface, mesh, obstacle_material, deformable_mesh)) =
        obstacles.get_mut(impact.target)
    else {
        return;
    };

    let dents_target =
        marks_per_impact(impact.damage, impact_assets).min(frame_budget.dents_remaining);
    let chips_target =
        chips_per_impact(impact.damage, impact_assets).min(frame_budget.chips_remaining);
    let sample_limit = dents_target;
    if sample_limit == 0 {
        return;
    }

    let normal = normalized_or_up(impact.normal);
    let world_to_local = obstacle_tf.affine().inverse();
    let samples = collect_new_impact_samples(
        impact.point,
        normal,
        impact.damage,
        impact_assets,
        &mut surface,
        &world_to_local,
        sample_limit,
    );
    if samples.is_empty() {
        return;
    }

    let Some(deformable_mesh) = deformable_mesh else {
        return;
    };

    let dents_applied = apply_dents_to_obstacle_mesh(
        meshes,
        &mesh.0,
        impact_assets,
        &samples,
        normal,
        &world_to_local,
        dents_target.min(samples.len()),
        deformable_mesh,
    );
    frame_budget.dents_remaining = frame_budget.dents_remaining.saturating_sub(dents_applied);
    if dents_applied == 0 {
        return;
    }

    if frame_budget.chips_remaining == 0 {
        return;
    }

    let chip_material = obstacle_material
        .map(|m| m.0.clone())
        .unwrap_or_else(|| impact_assets.chip_fallback_material.clone());
    let chips_to_spawn = chips_target.min(samples.len());

    for (idx, sample) in samples.iter().take(chips_to_spawn).enumerate() {
        spawn_debris_chip(
            commands,
            impact_assets,
            chip_material.clone(),
            sample.world_point,
            normal,
            idx as f32 + impact.damage,
        );
    }
    frame_budget.chips_remaining -= chips_to_spawn;
}

fn collect_new_impact_samples(
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

fn apply_dents_to_obstacle_mesh(
    meshes: &mut Assets<Mesh>,
    mesh_handle: &Handle<Mesh>,
    impact_assets: &ImpactAssets,
    samples: &[ImpactSample],
    impact_normal: Vec3,
    world_to_local: &Affine3A,
    dents_to_apply: usize,
    deformable_mesh: &DestructibleMesh,
) -> usize {
    if dents_to_apply == 0 {
        return 0;
    }

    let Some(mesh) = meshes.get_mut(mesh_handle) else {
        return 0;
    };

    let local_normal = normalized_or_up(world_to_local.transform_vector3(impact_normal));
    let applied_dents = {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            return 0;
        };

        let mut applied = 0usize;
        for sample in samples.iter().take(dents_to_apply) {
            let local_center = world_to_local.transform_point3(sample.world_point);
            let radius =
                (impact_assets.crater_size * sample.size_scale).max(impact_assets.base_web_radius);
            let depth = (impact_assets.crater_depth * sample.depth_scale)
                .min(deformable_mesh.max_dent_depth)
                .max(0.001);
            let normal_band = (radius * 0.45).max(0.01);

            let mut touched_any_vertex = false;
            for position in positions.iter_mut() {
                let vertex = Vec3::from_array(*position);
                let delta = vertex - local_center;
                let plane_distance = delta.dot(local_normal);
                if plane_distance.abs() > normal_band {
                    continue;
                }

                let tangent = delta - local_normal * plane_distance;
                let tangent_distance = tangent.length();
                if tangent_distance > radius {
                    continue;
                }

                let radial_falloff = 1.0 - tangent_distance / radius;
                let band_falloff = 1.0 - (plane_distance.abs() / normal_band);
                let requested = depth * radial_falloff.powi(2) * band_falloff.powi(2);
                let clamped_plane =
                    (plane_distance - requested).max(-deformable_mesh.max_dent_depth);
                let applied_depth = plane_distance - clamped_plane;
                if applied_depth <= f32::EPSILON {
                    continue;
                }

                let deformed = vertex - local_normal * applied_depth;
                *position = deformed.to_array();
                touched_any_vertex = true;
            }

            if touched_any_vertex {
                applied += 1;
            }
        }
        applied
    };

    if applied_dents > 0 {
        mesh.compute_smooth_normals();
        mesh.remove_attribute(Mesh::ATTRIBUTE_TANGENT);
    }

    applied_dents
}

fn spawn_debris_chip(
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

#[derive(Clone, Copy)]
struct WebMarkSample {
    offset: Vec2,
    size_scale: f32,
    depth_scale: f32,
}

fn sample_web_mark(
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

fn marks_per_impact(damage: f32, impact_assets: &ImpactAssets) -> usize {
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

fn chips_per_impact(damage: f32, impact_assets: &ImpactAssets) -> usize {
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

fn web_radius_for_damage(damage: f32, impact_assets: &ImpactAssets) -> f32 {
    let base = impact_assets.base_web_radius.max(0.01);
    let max = impact_assets.max_web_radius.max(base);
    let factor = if impact_assets.damage_for_max_web <= f32::EPSILON {
        1.0
    } else {
        (damage / impact_assets.damage_for_max_web).clamp(0.0, 1.0)
    };
    base + (max - base) * factor
}

fn normalized_or_up(v: Vec3) -> Vec3 {
    let n = v.normalize_or_zero();
    if n == Vec3::ZERO { Vec3::Y } else { n }
}

fn hash01(v: Vec3) -> f32 {
    let dot = v.dot(Vec3::new(12.9898, 78.233, 45.164));
    (dot.sin() * 43_758.547).fract().abs()
}
