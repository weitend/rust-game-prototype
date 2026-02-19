use bevy::prelude::*;

use crate::plugins::polygon::{
    config::PolygonConfig,
    layout::{SectionKind, SectionLayout},
};

use super::common::{section_center, section_span, spawn_visual_block};

pub fn spawn_section_layout_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &PolygonConfig,
    layout: &SectionLayout,
) {
    let reserved_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(232, 232, 232),
        unlit: true,
        ..default()
    });
    let control_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(188, 213, 240),
        unlit: true,
        ..default()
    });
    let movement_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(246, 209, 227),
        unlit: true,
        ..default()
    });
    let jump_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(214, 241, 204),
        unlit: true,
        ..default()
    });
    let collision_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(224, 214, 196),
        unlit: true,
        ..default()
    });
    let hitscan_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(231, 203, 186),
        unlit: true,
        ..default()
    });
    let cover_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(197, 223, 235),
        unlit: true,
        ..default()
    });
    let damage_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(234, 205, 205),
        unlit: true,
        ..default()
    });
    let vertical_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(206, 204, 236),
        unlit: true,
        ..default()
    });
    let stress_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(236, 228, 173),
        unlit: true,
        ..default()
    });
    let border_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(35, 35, 35),
        unlit: true,
        ..default()
    });

    for (col, row, section) in layout.iter() {
        if section == SectionKind::ControlHub {
            continue;
        }

        let center = config.module_center(col, row);
        let material = match section {
            SectionKind::Reserved => reserved_mat.clone(),
            SectionKind::ControlHub => control_mat.clone(),
            SectionKind::MovementCalibration => movement_mat.clone(),
            SectionKind::JumpAutostepLab => jump_mat.clone(),
            SectionKind::CollisionTorture => collision_mat.clone(),
            SectionKind::HitscanRange => hitscan_mat.clone(),
            SectionKind::CoverPeek => cover_mat.clone(),
            SectionKind::DamageTeamSandbox => damage_mat.clone(),
            SectionKind::VerticalCombat => vertical_mat.clone(),
            SectionKind::PerformanceStress => stress_mat.clone(),
        };

        spawn_visual_block(
            commands,
            meshes,
            &material,
            Vec3::new(center.x, 0.012, center.z),
            Vec3::new(config.module_size - 0.35, 0.02, config.module_size - 0.35),
        );
    }

    if let Some(center_bounds) = layout.bounds_of(SectionKind::ControlHub) {
        let center = section_center(config, center_bounds);
        let span = section_span(config, center_bounds);

        spawn_visual_block(
            commands,
            meshes,
            &control_mat,
            Vec3::new(center.x, 0.012, center.z),
            Vec3::new((span.x - 0.35).max(0.2), 0.02, (span.y - 0.35).max(0.2)),
        );
    }

    let span = config.platform_span();
    for idx in 0..=layout.grid() {
        let offset = -0.5 * span + idx as f32 * config.module_size;

        spawn_visual_block(
            commands,
            meshes,
            &border_mat,
            Vec3::new(0.0, 0.025, offset),
            Vec3::new(span, 0.03, 0.08),
        );
        spawn_visual_block(
            commands,
            meshes,
            &border_mat,
            Vec3::new(offset, 0.025, 0.0),
            Vec3::new(0.08, 0.03, span),
        );
    }
}
