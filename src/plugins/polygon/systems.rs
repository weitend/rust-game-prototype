use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::{
    config::PolygonConfig,
    layout::{SectionKind, SectionLayout},
    sections::{
        collision_torture::spawn_collision_torture, common::spawn_platform,
        control_hub::spawn_control_hub, cover_peek::spawn_cover_peek,
        damage_team_sandbox::spawn_damage_team_sandbox, hitscan_range::spawn_hitscan_range,
        jump_autostep_lab::spawn_jump_autostep_lab, layout_visuals::spawn_section_layout_visuals,
        movement_calibration::spawn_movement_calibration,
        performance_stress::spawn_performance_stress, vertical_combat::spawn_vertical_combat,
    },
};

pub fn setup_polygon_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<PolygonConfig>,
) {
    let config = config.sanitized();
    let layout = SectionLayout::default_for_grid(config.module_grid);

    let platform_material = create_checker_platform_material(&mut images, &mut materials, &config);

    spawn_platform(&mut commands, &mut meshes, platform_material, &config);
    spawn_section_layout_visuals(&mut commands, &mut meshes, &mut materials, &config, &layout);

    let hub_material = materials.add(Color::srgb_u8(90, 144, 212));
    let movement_material = materials.add(Color::srgb_u8(235, 120, 170));
    let jump_material = materials.add(Color::srgb_u8(126, 194, 108));
    let collision_material = materials.add(Color::srgb_u8(196, 171, 145));
    let hitscan_material = materials.add(Color::srgb_u8(215, 162, 128));
    let cover_material = materials.add(Color::srgb_u8(124, 171, 194));
    let damage_material = materials.add(Color::srgb_u8(184, 112, 112));
    let vertical_material = materials.add(Color::srgb_u8(142, 130, 214));
    let stress_material = materials.add(Color::srgb_u8(191, 182, 82));
    let guide_material = materials.add(Color::srgb_u8(245, 245, 245));
    let rail_material = materials.add(Color::srgb_u8(54, 58, 66));
    let wall_material = materials.add(Color::srgb_u8(79, 91, 110));
    let backstop_material = materials.add(Color::srgb_u8(102, 61, 40));
    let enemy_target_material = materials.add(Color::srgb_u8(208, 64, 64));
    let friendly_target_material = materials.add(Color::srgb_u8(54, 134, 212));

    if let Some(bounds) = layout.bounds_of(SectionKind::ControlHub) {
        spawn_control_hub(
            &mut commands,
            &mut meshes,
            &mut materials,
            &config,
            bounds,
            &hub_material,
            &guide_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::MovementCalibration) {
        spawn_movement_calibration(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &movement_material,
            &guide_material,
            &rail_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::JumpAutostepLab) {
        spawn_jump_autostep_lab(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &jump_material,
            &guide_material,
            &rail_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::CollisionTorture) {
        spawn_collision_torture(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &collision_material,
            &guide_material,
            &rail_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::HitscanRange) {
        spawn_hitscan_range(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &hitscan_material,
            &guide_material,
            &backstop_material,
            &enemy_target_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::CoverPeek) {
        spawn_cover_peek(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &cover_material,
            &guide_material,
            &wall_material,
            &enemy_target_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::DamageTeamSandbox) {
        spawn_damage_team_sandbox(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &damage_material,
            &wall_material,
            &enemy_target_material,
            &friendly_target_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::VerticalCombat) {
        spawn_vertical_combat(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &vertical_material,
            &wall_material,
            &enemy_target_material,
        );
    }

    if let Some(bounds) = layout.bounds_of(SectionKind::PerformanceStress) {
        spawn_performance_stress(
            &mut commands,
            &mut meshes,
            &config,
            bounds,
            &stress_material,
            &rail_material,
        );
    }
}

fn create_checker_platform_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    config: &PolygonConfig,
) -> Handle<StandardMaterial> {
    let span = config.platform_span();
    let uv_tiles = Vec2::new(span / config.tile_size, span / config.tile_size);

    let pink = [255, 192, 203, 255];
    let white = [255, 255, 255, 255];
    let checker_pixels = [
        pink[0], pink[1], pink[2], pink[3], white[0], white[1], white[2], white[3], white[0],
        white[1], white[2], white[3], pink[0], pink[1], pink[2], pink[3],
    ];

    let mut checker_image = Image::new_fill(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &checker_pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    checker_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::nearest()
    });

    let checker_texture = images.add(checker_image);

    materials.add(StandardMaterial {
        base_color_texture: Some(checker_texture),
        uv_transform: Affine2::from_scale(uv_tiles),
        ..default()
    })
}
