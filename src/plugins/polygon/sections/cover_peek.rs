use bevy::prelude::*;

use crate::{
    components::combat::Team,
    plugins::polygon::{config::PolygonConfig, layout::SectionBounds},
};

use super::common::{
    section_center, section_span, spawn_damage_dummy, spawn_static_block, spawn_visual_block,
};

pub fn spawn_cover_peek(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    cover_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
    wall_material: &Handle<StandardMaterial>,
    target_material: &Handle<StandardMaterial>,
) {
    let area_center = section_center(config, bounds);
    let area_span = section_span(config, bounds);

    let base_size = Vec3::new(
        (area_span.x - 1.5).max(10.0),
        0.24,
        (area_span.y - 1.5).max(10.0),
    );
    spawn_static_block(
        commands,
        meshes,
        cover_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        base_size,
        false,
    );

    // Cover lines: low/mid/high.
    let lane_z = [
        area_center.z - 0.22 * base_size.z,
        area_center.z,
        area_center.z + 0.22 * base_size.z,
    ];
    let cover_heights = [0.9, 1.4, 2.0];
    for (idx, z) in lane_z.into_iter().enumerate() {
        for col in 0..4 {
            let x = area_center.x - 0.32 * base_size.x + col as f32 * (0.2 * base_size.x);
            spawn_static_block(
                commands,
                meshes,
                cover_material,
                Vec3::new(x, 0.5 * cover_heights[idx], z),
                Vec3::new(2.3, cover_heights[idx], 1.0),
                true,
            );
        }
    }

    // Peek corners (L-shapes).
    let corner_x = area_center.x + 0.18 * base_size.x;
    for z in [
        area_center.z - 0.2 * base_size.z,
        area_center.z + 0.2 * base_size.z,
    ] {
        spawn_static_block(
            commands,
            meshes,
            wall_material,
            Vec3::new(corner_x, 1.2, z),
            Vec3::new(0.7, 2.4, 5.0),
            true,
        );
        spawn_static_block(
            commands,
            meshes,
            wall_material,
            Vec3::new(
                corner_x + 2.0,
                1.2,
                z + if z < area_center.z { 2.1 } else { -2.1 },
            ),
            Vec3::new(4.0, 2.4, 0.7),
            true,
        );
    }

    // Targets behind cover.
    for z in [
        area_center.z - 0.2 * base_size.z,
        area_center.z,
        area_center.z + 0.2 * base_size.z,
    ] {
        spawn_damage_dummy(
            commands,
            meshes,
            target_material,
            Vec3::new(area_center.x + 0.35 * base_size.x, 1.0, z),
            Vec3::new(0.9, 2.0, 0.9),
            Team::Enemy,
            100.0,
        );
    }

    // Visual midline for repeated strafe tests.
    spawn_visual_block(
        commands,
        meshes,
        guide_material,
        Vec3::new(area_center.x, 0.26, area_center.z),
        Vec3::new(base_size.x - 1.2, 0.03, 0.12),
    );

    let _ = config;
}
