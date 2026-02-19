use bevy::prelude::*;

use crate::plugins::polygon::{config::PolygonConfig, layout::SectionBounds};

use super::common::{section_center, section_span, spawn_static_block, spawn_visual_block};

pub fn spawn_collision_torture(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    collision_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
    rail_material: &Handle<StandardMaterial>,
) {
    let area_center = section_center(config, bounds);
    let area_span = section_span(config, bounds);

    let base_size = Vec3::new(
        (area_span.x - 1.5).max(8.0),
        0.24,
        (area_span.y - 1.5).max(8.0),
    );
    spawn_static_block(
        commands,
        meshes,
        collision_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        base_size,
        false,
    );

    // Perimeter rails.
    let rail_h = 1.2;
    let z_half = 0.5 * base_size.z;
    spawn_static_block(
        commands,
        meshes,
        rail_material,
        Vec3::new(area_center.x, rail_h * 0.5, area_center.z - z_half),
        Vec3::new(base_size.x, rail_h, 0.6),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        rail_material,
        Vec3::new(area_center.x, rail_h * 0.5, area_center.z + z_half),
        Vec3::new(base_size.x, rail_h, 0.6),
        true,
    );

    // Slalom walls with intentionally tight gaps.
    let wall_len = (base_size.z * 0.35).max(4.0);
    let wall_h = 1.8;
    let lane_left = area_center.x - base_size.x * 0.2;
    let lane_right = area_center.x + base_size.x * 0.2;
    let mut x = area_center.x - 0.36 * base_size.x;
    for idx in 0..8 {
        let z = if idx % 2 == 0 {
            area_center.z - 0.18 * base_size.z
        } else {
            area_center.z + 0.18 * base_size.z
        };
        let wall_x = if idx % 2 == 0 { lane_left } else { lane_right };
        spawn_static_block(
            commands,
            meshes,
            collision_material,
            Vec3::new(wall_x + x * 0.02, wall_h * 0.5, z),
            Vec3::new(0.7, wall_h, wall_len),
            true,
        );
        x += 0.1 * base_size.x;
    }

    // Low ceiling tunnel for collision edge checks.
    let tunnel_center = Vec3::new(area_center.x + 0.28 * base_size.x, 1.25, area_center.z);
    spawn_static_block(
        commands,
        meshes,
        guide_material,
        tunnel_center + Vec3::new(0.0, -0.65, -1.5),
        Vec3::new(0.45, 1.3, 3.4),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        guide_material,
        tunnel_center + Vec3::new(0.0, -0.65, 1.5),
        Vec3::new(0.45, 1.3, 3.4),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        rail_material,
        tunnel_center,
        Vec3::new(3.4, 0.35, 7.0),
        true,
    );

    // Visual center line to assist repeatable tests.
    spawn_visual_block(
        commands,
        meshes,
        guide_material,
        Vec3::new(area_center.x, 0.25, area_center.z),
        Vec3::new(base_size.x - 1.2, 0.03, 0.12),
    );
}
