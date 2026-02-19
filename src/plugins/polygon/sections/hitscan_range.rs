use bevy::prelude::*;

use crate::{
    components::combat::Team,
    plugins::polygon::{config::PolygonConfig, layout::SectionBounds},
};

use super::common::{
    section_center, section_span, spawn_damage_dummy, spawn_static_block, spawn_visual_block,
};

pub fn spawn_hitscan_range(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    range_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
    backstop_material: &Handle<StandardMaterial>,
    target_material: &Handle<StandardMaterial>,
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
        range_material,
        Vec3::new(area_center.x, 0.12, area_center.z),
        base_size,
        false,
    );

    // Backstop and side guards.
    let back_x = area_center.x + 0.5 * base_size.x - 1.0;
    spawn_static_block(
        commands,
        meshes,
        backstop_material,
        Vec3::new(back_x, 2.2, area_center.z),
        Vec3::new(1.8, 4.4, base_size.z),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        backstop_material,
        Vec3::new(area_center.x, 1.2, area_center.z - 0.5 * base_size.z),
        Vec3::new(base_size.x, 2.4, 0.6),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        backstop_material,
        Vec3::new(area_center.x, 1.2, area_center.z + 0.5 * base_size.z),
        Vec3::new(base_size.x, 2.4, 0.6),
        true,
    );

    // Distance strips.
    let start_x = area_center.x - 0.45 * base_size.x;
    let end_x = area_center.x + 0.35 * base_size.x;
    let stripes = 6;
    for idx in 0..=stripes {
        let t = idx as f32 / stripes as f32;
        let x = start_x + (end_x - start_x) * t;
        spawn_visual_block(
            commands,
            meshes,
            guide_material,
            Vec3::new(x, 0.26, area_center.z),
            Vec3::new(0.12, 0.03, base_size.z - 1.2),
        );
    }

    // Enemy dummies on three lanes and multiple distances.
    let lane_z_offsets = [-0.25 * base_size.z, 0.0, 0.25 * base_size.z];
    let distance_factors = [0.20, 0.35, 0.50, 0.65, 0.80, 0.90];
    for lane in lane_z_offsets {
        for factor in distance_factors {
            let x = start_x + (end_x - start_x) * factor;
            let z = area_center.z + lane;
            spawn_damage_dummy(
                commands,
                meshes,
                target_material,
                Vec3::new(x, 1.0, z),
                Vec3::new(0.9, 2.0, 0.9),
                Team::Enemy,
                100.0,
            );
        }
    }

    // Reference impact wall for quick tracer/impact checks.
    spawn_static_block(
        commands,
        meshes,
        backstop_material,
        Vec3::new(area_center.x + 0.08 * base_size.x, 1.2, area_center.z),
        Vec3::new(0.5, 2.4, 0.5 * base_size.z),
        true,
    );

    let _ = config;
}
