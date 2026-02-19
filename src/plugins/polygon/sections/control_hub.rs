use bevy::prelude::*;

use crate::plugins::polygon::{config::PolygonConfig, layout::SectionBounds};

use super::common::{section_center, section_span, spawn_static_block};

pub fn spawn_control_hub(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &PolygonConfig,
    bounds: SectionBounds,
    hub_material: &Handle<StandardMaterial>,
    guide_material: &Handle<StandardMaterial>,
) {
    let module = config.module_size;
    let center = section_center(config, bounds);
    let span = section_span(config, bounds);

    spawn_static_block(
        commands,
        meshes,
        hub_material,
        Vec3::new(center.x, 0.18, center.z),
        Vec3::new((span.x * 0.6).max(module), 0.36, (span.y * 0.6).max(module)),
        false,
    );
    spawn_static_block(
        commands,
        meshes,
        hub_material,
        Vec3::new(center.x, 0.1, center.z - module * 0.7),
        Vec3::new(module * 0.3, 0.2, module * 0.4),
        false,
    );

    let pad_offset = module * 0.35;
    let pad_positions = [
        Vec3::new(center.x - pad_offset, 0.12, center.z - pad_offset),
        Vec3::new(center.x + pad_offset, 0.12, center.z - pad_offset),
        Vec3::new(center.x - pad_offset, 0.12, center.z + pad_offset),
        Vec3::new(center.x + pad_offset, 0.12, center.z + pad_offset),
    ];
    for pos in pad_positions {
        spawn_static_block(
            commands,
            meshes,
            guide_material,
            pos,
            Vec3::new(module * 0.2, 0.24, module * 0.2),
            true,
        );
    }

    let north_marker = materials.add(Color::srgb_u8(240, 72, 72));
    let east_marker = materials.add(Color::srgb_u8(72, 160, 245));
    let south_marker = materials.add(Color::srgb_u8(72, 196, 124));
    let west_marker = materials.add(Color::srgb_u8(240, 190, 72));
    let marker_offset = module * 0.55;
    let orientation_markers = [
        (
            Vec3::new(center.x, 1.2, center.z - marker_offset),
            north_marker,
        ),
        (
            Vec3::new(center.x + marker_offset, 1.2, center.z),
            east_marker,
        ),
        (
            Vec3::new(center.x, 1.2, center.z + marker_offset),
            south_marker,
        ),
        (
            Vec3::new(center.x - marker_offset, 1.2, center.z),
            west_marker,
        ),
    ];

    for (pos, material) in orientation_markers {
        spawn_static_block(
            commands,
            meshes,
            &material,
            pos,
            Vec3::new(0.9, 2.4, 0.9),
            true,
        );
    }
}
