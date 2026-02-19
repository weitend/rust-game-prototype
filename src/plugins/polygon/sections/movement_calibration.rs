use bevy::prelude::*;

use crate::plugins::polygon::{config::PolygonConfig, layout::SectionBounds};

use super::common::{section_center, section_span, spawn_static_block, spawn_visual_block};

pub fn spawn_movement_calibration(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    movement_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
    rail_material: &Handle<StandardMaterial>,
) {
    let section_center = section_center(config, bounds);
    let section_span = section_span(config, bounds);
    let track_depth = (section_span.y * 0.9).max(config.module_size * 0.8);
    let track_width = (section_span.x - 2.0).max(config.module_size * 2.0);

    spawn_static_block(
        commands,
        meshes,
        movement_material,
        Vec3::new(section_center.x, 0.12, section_center.z),
        Vec3::new(track_width, 0.24, track_depth),
        false,
    );

    let rail_z_offset = track_depth * 0.49;
    for z in [
        section_center.z - rail_z_offset,
        section_center.z + rail_z_offset,
    ] {
        spawn_static_block(
            commands,
            meshes,
            rail_material,
            Vec3::new(section_center.x, 0.35, z),
            Vec3::new(track_width, 0.7, 0.6),
            true,
        );
    }

    let lane_offsets = [-0.25, -0.075, 0.10, 0.275];
    for offset in lane_offsets {
        spawn_visual_block(
            commands,
            meshes,
            guide_material,
            Vec3::new(
                section_center.x,
                0.26,
                section_center.z + offset * config.module_size,
            ),
            Vec3::new(track_width - 2.0, 0.03, 0.12),
        );
    }

    let marker_limit = 0.5 * track_width - 4.0;
    let mut marker_x = -marker_limit;
    while marker_x <= marker_limit {
        for z in [
            section_center.z - rail_z_offset + 1.8,
            section_center.z + rail_z_offset - 1.8,
        ] {
            spawn_static_block(
                commands,
                meshes,
                guide_material,
                Vec3::new(marker_x, 0.5, z),
                Vec3::new(0.25, 1.0, 0.25),
                true,
            );
        }

        marker_x += 5.0;
    }

    let step_heights = [0.10, 0.18, 0.25, 0.30, 0.38, 0.46];
    let step_start = section_center.x - 0.42 * section_span.x;
    let step_spacing = 0.03 * section_span.x;
    for (idx, height) in step_heights.iter().enumerate() {
        let x = step_start + idx as f32 * step_spacing;
        spawn_static_block(
            commands,
            meshes,
            movement_material,
            Vec3::new(x, 0.5 * *height, section_center.z),
            Vec3::new(2.0, *height, 3.0),
            true,
        );
    }

    let jump_pad_x_factors = [0.225, 0.29375, 0.375, 0.45];
    for factor in jump_pad_x_factors {
        let x = section_center.x + factor * section_span.x;
        spawn_static_block(
            commands,
            meshes,
            movement_material,
            Vec3::new(x, 0.16, section_center.z),
            Vec3::new(3.2, 0.32, 3.2),
            true,
        );
    }

    let slalom_positions = [
        Vec3::new(
            section_center.x - 0.125 * section_span.x,
            0.9,
            section_center.z + 0.2 * config.module_size,
        ),
        Vec3::new(
            section_center.x - 0.075 * section_span.x,
            0.9,
            section_center.z - 0.15 * config.module_size,
        ),
        Vec3::new(
            section_center.x - 0.025 * section_span.x,
            0.9,
            section_center.z + 0.2 * config.module_size,
        ),
        Vec3::new(
            section_center.x + 0.025 * section_span.x,
            0.9,
            section_center.z - 0.15 * config.module_size,
        ),
        Vec3::new(
            section_center.x + 0.075 * section_span.x,
            0.9,
            section_center.z + 0.2 * config.module_size,
        ),
        Vec3::new(
            section_center.x + 0.125 * section_span.x,
            0.9,
            section_center.z - 0.15 * config.module_size,
        ),
    ];
    for pos in slalom_positions {
        spawn_static_block(
            commands,
            meshes,
            rail_material,
            pos,
            Vec3::new(0.8, 1.8, 0.8),
            true,
        );
    }
}
