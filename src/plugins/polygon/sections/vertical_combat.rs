use bevy::prelude::*;

use crate::{
    components::combat::Team,
    plugins::polygon::{config::PolygonConfig, layout::SectionBounds},
};

use super::common::{section_center, section_span, spawn_damage_dummy, spawn_static_block};

pub fn spawn_vertical_combat(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    platform_material: &Handle<StandardMaterial>,
    structure_material: &Handle<StandardMaterial>,
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
        platform_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        base_size,
        false,
    );

    // Three elevation towers.
    let tower_positions = [
        Vec3::new(
            area_center.x - 0.28 * base_size.x,
            2.0,
            area_center.z - 0.2 * base_size.z,
        ),
        Vec3::new(area_center.x, 2.8, area_center.z + 0.1 * base_size.z),
        Vec3::new(
            area_center.x + 0.28 * base_size.x,
            3.6,
            area_center.z - 0.15 * base_size.z,
        ),
    ];

    for (idx, pos) in tower_positions.into_iter().enumerate() {
        let h = 4.0 + idx as f32 * 1.4;
        spawn_static_block(
            commands,
            meshes,
            structure_material,
            Vec3::new(pos.x, h * 0.5, pos.z),
            Vec3::new(4.0, h, 4.0),
            true,
        );

        spawn_damage_dummy(
            commands,
            meshes,
            target_material,
            Vec3::new(pos.x, h + 1.0, pos.z),
            Vec3::new(0.9, 2.0, 0.9),
            Team::Enemy,
            100.0,
        );
    }

    // Stair/ramp-style blocks to reach upper positions.
    for idx in 0..6 {
        let x = area_center.x - 0.4 * base_size.x + idx as f32 * (0.12 * base_size.x);
        let y = 0.2 + idx as f32 * 0.35;
        spawn_static_block(
            commands,
            meshes,
            platform_material,
            Vec3::new(x, y, area_center.z + 0.28 * base_size.z),
            Vec3::new(2.2, 0.3, 2.2),
            true,
        );
    }

    // Bridge between center and right tower.
    spawn_static_block(
        commands,
        meshes,
        structure_material,
        Vec3::new(
            area_center.x + 0.14 * base_size.x,
            4.5,
            area_center.z - 0.02 * base_size.z,
        ),
        Vec3::new(7.0, 0.4, 2.0),
        true,
    );

    let _ = config;
}
