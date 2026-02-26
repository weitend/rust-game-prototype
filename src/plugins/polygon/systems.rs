use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::components::obstacle::{Obstacle, ObstacleNetId};

use super::{
    config::PolygonConfig,
    layout::{SectionKind, SectionLayout},
    sections::{
        collision_torture::spawn_collision_torture,
        common::{section_center, section_span, spawn_platform},
        cover_peek::spawn_cover_peek,
        damage_team_sandbox::spawn_damage_team_sandbox,
        hitscan_range::spawn_hitscan_range,
        jump_autostep_lab::spawn_jump_autostep_lab,
        layout_visuals::spawn_section_layout_visuals,
        movement_calibration::spawn_movement_calibration,
        performance_stress::spawn_performance_stress,
        vertical_combat::spawn_vertical_combat,
    },
    teleports::spawn_teleport_pad,
};

pub fn setup_polygon_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Option<Res<AssetServer>>,
    fonts: Option<Res<Assets<Font>>>,
    config: Res<PolygonConfig>,
) {
    let config = config.sanitized();
    let layout = SectionLayout::default_for_grid(config.module_grid);

    let platform_material = create_checker_platform_material(&mut images, &mut materials, &config);

    spawn_platform(&mut commands, &mut meshes, platform_material, &config);
    spawn_polygon_lighting(&mut commands, &config);
    spawn_section_layout_visuals(&mut commands, &mut meshes, &mut materials, &config, &layout);

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

    if let (Some(asset_server), Some(_)) = (asset_server, fonts) {
        let teleport_font = asset_server.load("fonts/arial_unicode.ttf");
        spawn_teleport_network(
            &mut commands,
            &mut meshes,
            &mut materials,
            &teleport_font,
            &config,
            &layout,
        );
    }
}

pub fn assign_obstacle_net_ids_system(
    mut commands: Commands,
    obstacles: Query<(Entity, &Transform), (With<Obstacle>, Without<ObstacleNetId>)>,
) {
    let mut entries: Vec<(Entity, Vec3)> = obstacles
        .iter()
        .map(|(entity, transform)| (entity, transform.translation))
        .collect();
    if entries.is_empty() {
        return;
    }

    entries.sort_by(|(left_entity, left_pos), (right_entity, right_pos)| {
        left_pos
            .x
            .total_cmp(&right_pos.x)
            .then(left_pos.z.total_cmp(&right_pos.z))
            .then(left_pos.y.total_cmp(&right_pos.y))
            .then(left_entity.index().cmp(&right_entity.index()))
    });

    for (idx, (entity, _)) in entries.into_iter().enumerate() {
        commands
            .entity(entity)
            .insert(ObstacleNetId(idx as u64 + 1));
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

fn spawn_polygon_lighting(commands: &mut Commands, config: &PolygonConfig) {
    let span = config.platform_span();
    let height = (0.52 * span).max(24.0);

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.68, 0.71, 0.78),
        brightness: 38.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 2_400.0,
            color: Color::srgb(1.0, 0.97, 0.92),
            ..default()
        },
        Transform::from_xyz(0.0, height, 0.0)
            .looking_at(Vec3::new(0.22 * span, 0.0, -0.18 * span), Vec3::Y),
    ));

    let step = if config.module_grid >= 6 {
        3
    } else if config.module_grid >= 4 {
        2
    } else {
        1
    };
    let points_per_axis = (config.module_grid + step - 1) / step;
    let point_count = (points_per_axis * points_per_axis) as f32;
    let point_intensity = (70_000.0 / point_count).clamp(4_500.0, 11_000.0);
    let point_height = (0.30 * span).max(16.0);
    let range = (config.module_size * (step as f32 + 0.25)).max(22.0);

    for row in (0..config.module_grid).step_by(step) {
        for col in (0..config.module_grid).step_by(step) {
            let center = config.module_center(col, row);
            commands.spawn((
                PointLight {
                    intensity: point_intensity,
                    range,
                    color: Color::srgb(0.92, 0.94, 1.0),
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(center.x, point_height, center.z),
            ));
        }
    }
}

fn spawn_teleport_network(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    font: &Handle<Font>,
    config: &PolygonConfig,
    layout: &SectionLayout,
) {
    let Some(hub_bounds) = layout.bounds_of(SectionKind::ControlHub) else {
        return;
    };
    let hub_center = section_center(config, hub_bounds);
    let hub_span = section_span(config, hub_bounds);
    let destination_specs = [
        (
            SectionKind::MovementCalibration,
            "E: В полигон \"Движение\"",
            Color::srgb(0.83, 0.47, 0.62),
        ),
        (
            SectionKind::HitscanRange,
            "E: В полигон \"Стрельба\"",
            Color::srgb(0.82, 0.57, 0.47),
        ),
        (
            SectionKind::CollisionTorture,
            "E: В полигон \"Коллизии\"",
            Color::srgb(0.75, 0.62, 0.49),
        ),
        (
            SectionKind::CoverPeek,
            "E: В полигон \"Укрытия\"",
            Color::srgb(0.48, 0.66, 0.75),
        ),
        (
            SectionKind::DamageTeamSandbox,
            "E: В полигон \"Урон/Команды\"",
            Color::srgb(0.72, 0.46, 0.46),
        ),
        (
            SectionKind::VerticalCombat,
            "E: В полигон \"Вертикальный бой\"",
            Color::srgb(0.54, 0.49, 0.78),
        ),
        (
            SectionKind::JumpAutostepLab,
            "E: В полигон \"Прыжки/Автошаг\"",
            Color::srgb(0.45, 0.68, 0.43),
        ),
        (
            SectionKind::PerformanceStress,
            "E: В полигон \"Нагрузка\"",
            Color::srgb(0.72, 0.67, 0.35),
        ),
    ];

    let active_count = destination_specs
        .iter()
        .filter(|(kind, _, _)| layout.bounds_of(*kind).is_some())
        .count();
    if active_count == 0 {
        return;
    }

    let cols = 4usize.min(active_count.max(1));
    let rows = (active_count + cols - 1) / cols;
    let x_step = (hub_span.x * 0.72 / cols.saturating_sub(1).max(1) as f32).max(4.6);
    let z_step = (hub_span.y * 0.35 / rows.max(1) as f32).max(4.4);
    let start_x = hub_center.x - 0.5 * x_step * cols.saturating_sub(1) as f32;
    let start_z = hub_center.z + 0.28 * hub_span.y;

    let mut idx = 0usize;
    for (kind, hub_label, hub_color) in destination_specs {
        let Some(bounds) = layout.bounds_of(kind) else {
            continue;
        };

        let row = idx / cols;
        let col = idx % cols;
        idx += 1;

        let hub_pad_pos = Vec3::new(
            start_x + col as f32 * x_step,
            0.16,
            start_z - row as f32 * z_step,
        );
        let hub_pad_landing = Vec3::new(hub_pad_pos.x, 0.95, hub_pad_pos.z);

        let target_center = section_center(config, bounds);
        let target_span = section_span(config, bounds);
        let offset = section_return_offset(kind, target_span);
        let return_pad_pos =
            Vec3::new(target_center.x + offset.x, 0.16, target_center.z + offset.y);
        let return_pad_landing = Vec3::new(return_pad_pos.x, 0.95, return_pad_pos.z);

        spawn_teleport_pad(
            commands,
            meshes,
            materials,
            font,
            hub_pad_pos,
            return_pad_landing,
            hub_label,
            hub_color,
        );

        let return_label = format!("E: В хаб ({})", section_short_name(kind));
        spawn_teleport_pad(
            commands,
            meshes,
            materials,
            font,
            return_pad_pos,
            hub_pad_landing,
            &return_label,
            Color::srgb(0.37, 0.48, 0.63),
        );
    }
}

fn section_return_offset(kind: SectionKind, span: Vec2) -> Vec2 {
    match kind {
        SectionKind::MovementCalibration | SectionKind::HitscanRange => {
            Vec2::new(0.0, 0.40 * span.y)
        }
        SectionKind::JumpAutostepLab | SectionKind::PerformanceStress => {
            Vec2::new(0.0, -0.40 * span.y)
        }
        SectionKind::CollisionTorture | SectionKind::DamageTeamSandbox => {
            Vec2::new(0.40 * span.x, 0.0)
        }
        SectionKind::CoverPeek | SectionKind::VerticalCombat => Vec2::new(-0.40 * span.x, 0.0),
        _ => Vec2::ZERO,
    }
}

fn section_short_name(kind: SectionKind) -> &'static str {
    match kind {
        SectionKind::MovementCalibration => "движение",
        SectionKind::HitscanRange => "стрельба",
        SectionKind::CollisionTorture => "коллизии",
        SectionKind::CoverPeek => "укрытия",
        SectionKind::DamageTeamSandbox => "урон/команды",
        SectionKind::VerticalCombat => "вертикаль",
        SectionKind::JumpAutostepLab => "прыжки",
        SectionKind::PerformanceStress => "нагрузка",
        SectionKind::ControlHub => "хаб",
        SectionKind::Reserved => "сектор",
    }
}
