use bevy::prelude::*;

use crate::{
    components::{
        destructible_mesh::DestructibleMesh, destructible_surface::DestructibleSurface,
        obstacle::Obstacle,
    },
    resources::impact_assets::ImpactAssets,
    utils::impact_math::{chips_per_impact, marks_per_impact, normalized_or_up},
};

use super::{
    super::ImpactEvent,
    debris::spawn_debris_chip,
    deform::apply_dents_to_obstacle_mesh,
    sampling::collect_new_impact_samples,
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

pub fn process_impact_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut impact_events: MessageReader<ImpactEvent>,
    mut obstacles: ObstacleImpactQuery<'_, '_>,
    impact_assets: Res<ImpactAssets>,
) {
    let mut frame_budget = ImpactFrameBudget {
        dents_remaining: impact_assets.max_marks_per_frame,
        chips_remaining: impact_assets.max_chips_per_frame,
    };

    for impact in impact_events.read() {
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

