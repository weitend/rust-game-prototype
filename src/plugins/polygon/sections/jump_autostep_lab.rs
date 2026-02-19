use bevy::prelude::*;

use crate::plugins::polygon::{config::PolygonConfig, layout::SectionBounds};

use super::common::{section_center, section_span, spawn_static_block, spawn_visual_block};

pub fn spawn_jump_autostep_lab(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    jump_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
    rail_material: &Handle<StandardMaterial>,
) {
    let area_center = section_center(config, bounds);
    let area_span = section_span(config, bounds);
    let lab_depth = (area_span.y * 0.9).max(config.module_size * 0.8);
    let lane_width = (area_span.x - 2.0).max(config.module_size * 2.0);

    // Main base for the lab.
    spawn_static_block(
        commands,
        meshes,
        jump_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        Vec3::new(lane_width, 0.24, lab_depth),
        false,
    );

    // Side rails keep player inside the lab lanes.
    let rail_z = lab_depth * 0.49;
    for z in [area_center.z - rail_z, area_center.z + rail_z] {
        spawn_static_block(
            commands,
            meshes,
            rail_material,
            Vec3::new(area_center.x, 0.35, z),
            Vec3::new(lane_width, 0.7, 0.6),
            true,
        );
    }

    // Lane separators.
    let lane_offsets = [-0.22, 0.0, 0.22];
    for offset in lane_offsets {
        spawn_visual_block(
            commands,
            meshes,
            guide_material,
            Vec3::new(
                area_center.x,
                0.26,
                area_center.z + offset * config.module_size,
            ),
            Vec3::new(lane_width - 2.0, 0.03, 0.12),
        );
    }

    spawn_autostep_threshold_lane(
        commands,
        meshes,
        config,
        jump_material,
        area_center,
        area_span,
    );
    spawn_jump_gap_lane(
        commands,
        meshes,
        jump_material,
        guide_material,
        area_center,
        area_span,
    );
    spawn_precision_steps_lane(
        commands,
        meshes,
        jump_material,
        area_center,
        area_span,
        rail_material,
    );
}

fn spawn_autostep_threshold_lane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    jump_material: &Handle<StandardMaterial>,
    area_center: Vec3,
    area_span: Vec2,
) {
    // Tests around KCC autostep max_height=0.25.
    let heights = [0.10, 0.18, 0.23, 0.25, 0.28, 0.34, 0.42];
    let start_x = area_center.x - 0.43 * area_span.x;
    let spacing = 0.043 * area_span.x;
    let lane_z = area_center.z - 0.22 * config.module_size;

    for (idx, height) in heights.iter().enumerate() {
        let x = start_x + idx as f32 * spacing;
        spawn_static_block(
            commands,
            meshes,
            jump_material,
            Vec3::new(x, 0.5 * *height, lane_z),
            Vec3::new(2.2, *height, 2.8),
            true,
        );
    }
}

fn spawn_jump_gap_lane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    jump_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
    area_center: Vec3,
    area_span: Vec2,
) {
    let lane_z = area_center.z;

    // Sequential platforms with increasing gap width.
    let platform_x = [-0.44, -0.32, -0.17, 0.00, 0.22, 0.47];
    let platform_y = [0.12, 0.18, 0.24, 0.28, 0.34, 0.44];

    for (factor, height) in platform_x.into_iter().zip(platform_y) {
        let x = area_center.x + factor * area_span.x;
        spawn_static_block(
            commands,
            meshes,
            jump_material,
            Vec3::new(x, height, lane_z),
            Vec3::new(3.4, 0.24, 3.4),
            true,
        );
    }

    // Landing strip at the end of the jump lane.
    spawn_static_block(
        commands,
        meshes,
        guide_material,
        Vec3::new(area_center.x + 0.56 * area_span.x, 0.1, lane_z),
        Vec3::new(4.0, 0.2, 4.0),
        true,
    );
}

fn spawn_precision_steps_lane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    jump_material: &Handle<StandardMaterial>,
    area_center: Vec3,
    area_span: Vec2,
    rail_material: &Handle<StandardMaterial>,
) {
    let lane_z = area_center.z + 0.22 * (area_span.y * 0.5);

    // Narrow staircase for collision and landing precision.
    for idx in 0..8 {
        let x = area_center.x - 0.30 * area_span.x + idx as f32 * (0.055 * area_span.x);
        let step_height = 0.08 + idx as f32 * 0.06;

        spawn_static_block(
            commands,
            meshes,
            jump_material,
            Vec3::new(x, 0.5 * step_height, lane_z),
            Vec3::new(1.2, step_height, 1.2),
            true,
        );
    }

    // A final wall to force turnaround and repeated attempts.
    spawn_static_block(
        commands,
        meshes,
        rail_material,
        Vec3::new(area_center.x + 0.2 * area_span.x, 1.1, lane_z),
        Vec3::new(0.8, 2.2, 4.0),
        true,
    );
}
