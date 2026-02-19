use bevy::prelude::*;

use crate::{
    components::combat::Team,
    plugins::polygon::{config::PolygonConfig, layout::SectionBounds},
};

use super::common::{section_center, section_span, spawn_damage_dummy, spawn_static_block};

pub fn spawn_damage_team_sandbox(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    arena_material: &Handle<StandardMaterial>,
    divider_material: &Handle<StandardMaterial>,
    enemy_target_material: &Handle<StandardMaterial>,
    friendly_target_material: &Handle<StandardMaterial>,
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
        arena_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        base_size,
        false,
    );

    // Central divider for quick left/right team checks.
    spawn_static_block(
        commands,
        meshes,
        divider_material,
        Vec3::new(area_center.x, 1.1, area_center.z),
        Vec3::new(0.7, 2.2, base_size.z - 1.2),
        true,
    );

    // Enemy team lane (left from spawn perspective).
    let left_x = area_center.x - 0.24 * base_size.x;
    for idx in 0..6 {
        let z = area_center.z - 0.3 * base_size.z + idx as f32 * (0.12 * base_size.z);
        spawn_damage_dummy(
            commands,
            meshes,
            enemy_target_material,
            Vec3::new(left_x, 1.0, z),
            Vec3::new(0.9, 2.0, 0.9),
            Team::Enemy,
            100.0,
        );
    }

    // Friendly team lane to validate friendly-fire rules.
    let right_x = area_center.x + 0.24 * base_size.x;
    for idx in 0..6 {
        let z = area_center.z - 0.3 * base_size.z + idx as f32 * (0.12 * base_size.z);
        spawn_damage_dummy(
            commands,
            meshes,
            friendly_target_material,
            Vec3::new(right_x, 1.0, z),
            Vec3::new(0.9, 2.0, 0.9),
            Team::Player,
            100.0,
        );
    }

    // Backstop at end of arena.
    spawn_static_block(
        commands,
        meshes,
        divider_material,
        Vec3::new(area_center.x + 0.42 * base_size.x, 1.6, area_center.z),
        Vec3::new(0.9, 3.2, base_size.z),
        true,
    );

    let _ = config;
}
