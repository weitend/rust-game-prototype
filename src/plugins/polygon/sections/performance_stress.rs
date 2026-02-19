use bevy::prelude::*;

use crate::plugins::polygon::{config::PolygonConfig, layout::SectionBounds};

use super::common::{section_center, section_span, spawn_static_block};

pub fn spawn_performance_stress(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    floor_material: &Handle<StandardMaterial>,
    stress_material: &Handle<StandardMaterial>,
) {
    let area_center = section_center(config, bounds);
    let area_span = section_span(config, bounds);

    let base_size = Vec3::new(
        (area_span.x - 1.5).max(12.0),
        0.24,
        (area_span.y - 1.5).max(10.0),
    );
    spawn_static_block(
        commands,
        meshes,
        floor_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        base_size,
        false,
    );

    // Dense obstacle grid (render + collider stress).
    let cols = 14;
    let rows = 8;
    for row in 0..rows {
        for col in 0..cols {
            let tx = if cols > 1 {
                col as f32 / (cols - 1) as f32
            } else {
                0.5
            };
            let tz = if rows > 1 {
                row as f32 / (rows - 1) as f32
            } else {
                0.5
            };

            let x = area_center.x - 0.42 * base_size.x + tx * (0.84 * base_size.x);
            let z = area_center.z - 0.35 * base_size.z + tz * (0.70 * base_size.z);
            let height = 0.8 + ((row + col) % 5) as f32 * 0.35;

            spawn_static_block(
                commands,
                meshes,
                stress_material,
                Vec3::new(x, 0.5 * height, z),
                Vec3::new(0.85, height, 0.85),
                true,
            );
        }
    }

    // Extra heavy wall strips.
    for idx in 0..4 {
        let z = area_center.z - 0.28 * base_size.z + idx as f32 * (0.19 * base_size.z);
        spawn_static_block(
            commands,
            meshes,
            stress_material,
            Vec3::new(area_center.x, 1.5, z),
            Vec3::new(base_size.x - 3.5, 3.0, 0.35),
            true,
        );
    }

    let _ = config;
}
