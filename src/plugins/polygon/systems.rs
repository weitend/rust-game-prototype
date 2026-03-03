use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    mesh::Indices,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Friction, Group, RigidBody};

use crate::components::{
    ground_surface::{GroundSurfaceKind, GroundSurfaceTag},
    obstacle::{Obstacle, ObstacleNetId},
};
use crate::resources::ground_surface_visual_catalog::GroundSurfaceVisualCatalog;
use crate::utils::collision_groups::GROUP_WORLD;

use super::{
    config::{PolygonConfig, PolygonMapMode},
    layout::{SectionKind, SectionLayout},
    sections::{
        collision_torture::spawn_collision_torture,
        common::{section_center, section_span, spawn_platform, spawn_static_block},
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
    ground_visual_catalog: Res<GroundSurfaceVisualCatalog>,
) {
    let config = config.sanitized();
    if matches!(config.map_mode, PolygonMapMode::HillsDemo) {
        setup_hills_demo_system(
            &mut commands,
            &mut images,
            &mut meshes,
            &mut materials,
            asset_server.as_deref(),
            &ground_visual_catalog,
            &config,
        );
        return;
    }

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

fn setup_hills_demo_system(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: Option<&AssetServer>,
    ground_visual_catalog: &GroundSurfaceVisualCatalog,
    config: &PolygonConfig,
) {
    spawn_polygon_lighting(commands, config);
    spawn_hills_terrain(
        commands,
        images,
        meshes,
        materials,
        asset_server,
        ground_visual_catalog,
        config,
    );
    spawn_hills_surface_zones(commands, meshes, materials, config);
    spawn_hills_landmarks(commands, meshes, materials, config);
}

fn spawn_hills_terrain(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: Option<&AssetServer>,
    ground_visual_catalog: &GroundSurfaceVisualCatalog,
    config: &PolygonConfig,
) {
    let resolution = config.hills_resolution;
    let span = config.platform_span();
    let half = 0.5 * span;
    let step = span / resolution as f32;
    let verts_per_axis = resolution + 1;

    let mut positions = Vec::<[f32; 3]>::with_capacity(verts_per_axis * verts_per_axis);
    let mut normals = Vec::<[f32; 3]>::with_capacity(verts_per_axis * verts_per_axis);
    let mut uvs = Vec::<[f32; 2]>::with_capacity(verts_per_axis * verts_per_axis);
    let mut mesh_indices = Vec::<u32>::with_capacity(resolution * resolution * 6);
    let mut collider_indices = Vec::<[u32; 3]>::with_capacity(resolution * resolution * 2);

    for row in 0..=resolution {
        let z = -half + row as f32 * step;
        let v = row as f32 / resolution as f32;
        for col in 0..=resolution {
            let x = -half + col as f32 * step;
            let u = col as f32 / resolution as f32;
            let y = sample_hills_height(x, z, config);

            positions.push([x, y, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, v]);
        }
    }

    for row in 0..resolution {
        for col in 0..resolution {
            let i0 = (row * verts_per_axis + col) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + verts_per_axis as u32;
            let i3 = i2 + 1;

            mesh_indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
            collider_indices.push([i0, i2, i1]);
            collider_indices.push([i1, i2, i3]);
        }
    }

    let collider_vertices: Vec<Vec3> = positions
        .iter()
        .map(|p| Vec3::new(p[0], p[1], p[2]))
        .collect();

    let mut terrain_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    terrain_mesh.insert_indices(Indices::U32(mesh_indices));
    terrain_mesh.compute_smooth_normals();

    let terrain_mesh_handle = meshes.add(terrain_mesh);
    let terrain_material =
        create_hills_terrain_material(images, materials, asset_server, ground_visual_catalog);

    let terrain_collider = match Collider::trimesh(collider_vertices, collider_indices) {
        Ok(collider) => collider,
        Err(err) => {
            warn!("Failed to build hills terrain collider: {err}");
            return;
        }
    };

    commands.spawn((
        Name::new("HillsTerrain"),
        Mesh3d(terrain_mesh_handle),
        MeshMaterial3d(terrain_material),
        Transform::default(),
        RigidBody::Fixed,
        terrain_collider,
        CollisionGroups::new(GROUP_WORLD, Group::ALL),
        Friction::coefficient(1.0),
        GroundSurfaceTag::new(GroundSurfaceKind::Grass),
    ));
}

fn create_hills_terrain_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: Option<&AssetServer>,
    ground_visual_catalog: &GroundSurfaceVisualCatalog,
) -> Handle<StandardMaterial> {
    let Some(asset_server) = asset_server else {
        return create_checker_hills_material(images, materials);
    };

    let Some(texture_set) = ground_visual_catalog.terrain_texture_set_for(GroundSurfaceKind::Grass)
    else {
        return create_checker_hills_material(images, materials);
    };

    materials.add(StandardMaterial {
        base_color_texture: Some(load_tiled_texture(
            asset_server,
            texture_set.base_color_path,
            true,
        )),
        normal_map_texture: texture_set
            .normal_map_path
            .map(|path| load_tiled_texture(asset_server, path, false)),
        metallic_roughness_texture: texture_set
            .metallic_roughness_map_path
            .map(|path| load_tiled_texture(asset_server, path, false)),
        occlusion_texture: texture_set
            .occlusion_map_path
            .map(|path| load_tiled_texture(asset_server, path, false)),
        uv_transform: Affine2::from_scale(texture_set.uv_tiling),
        perceptual_roughness: texture_set.perceptual_roughness,
        ..default()
    })
}

fn load_tiled_texture(
    asset_server: &AssetServer,
    path: &'static str,
    is_srgb: bool,
) -> Handle<Image> {
    let sampler = repeating_linear_sampler();
    asset_server.load_with_settings(path, move |settings: &mut ImageLoaderSettings| {
        settings.is_srgb = is_srgb;
        settings.sampler = sampler.clone();
    })
}

fn repeating_linear_sampler() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    })
}

fn spawn_hills_landmarks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &PolygonConfig,
) {
    let rock_material = materials.add(Color::srgb_u8(108, 106, 98));
    let wall_material = materials.add(Color::srgb_u8(77, 82, 92));
    let span = config.platform_span();
    let half = 0.5 * span + 2.0;

    spawn_static_block(
        commands,
        meshes,
        &rock_material,
        Vec3::new(-18.0, 2.2, -26.0),
        Vec3::new(7.0, 4.4, 9.0),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        &rock_material,
        Vec3::new(23.0, 3.0, -4.0),
        Vec3::new(9.0, 6.0, 8.0),
        true,
    );
    spawn_static_block(
        commands,
        meshes,
        &rock_material,
        Vec3::new(-6.0, 2.8, 22.0),
        Vec3::new(11.0, 5.6, 6.0),
        true,
    );

    let wall_height = 8.0;
    let wall_thickness = 2.0;
    spawn_static_block(
        commands,
        meshes,
        &wall_material,
        Vec3::new(0.0, wall_height * 0.5 - 0.2, -half),
        Vec3::new(span + 10.0, wall_height, wall_thickness),
        false,
    );
    spawn_static_block(
        commands,
        meshes,
        &wall_material,
        Vec3::new(0.0, wall_height * 0.5 - 0.2, half),
        Vec3::new(span + 10.0, wall_height, wall_thickness),
        false,
    );
    spawn_static_block(
        commands,
        meshes,
        &wall_material,
        Vec3::new(-half, wall_height * 0.5 - 0.2, 0.0),
        Vec3::new(wall_thickness, wall_height, span + 10.0),
        false,
    );
    spawn_static_block(
        commands,
        meshes,
        &wall_material,
        Vec3::new(half, wall_height * 0.5 - 0.2, 0.0),
        Vec3::new(wall_thickness, wall_height, span + 10.0),
        false,
    );
}

fn spawn_hills_surface_zones(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &PolygonConfig,
) {
    let zone_size = Vec3::new(5.2, 0.35, 8.0);
    let zone_z = 12.0;
    let zone_top_offset = 0.10;
    let zone_mesh = meshes.add(Cuboid::new(zone_size.x, zone_size.y, zone_size.z));

    let zone_specs = [
        (
            -12.0_f32,
            GroundSurfaceKind::Default,
            Color::srgb_u8(196, 196, 196),
            "SurfaceZone::Default",
        ),
        (
            -6.0,
            GroundSurfaceKind::Grass,
            Color::srgb_u8(96, 168, 102),
            "SurfaceZone::Grass",
        ),
        (
            0.0,
            GroundSurfaceKind::Mud,
            Color::srgb_u8(145, 102, 72),
            "SurfaceZone::Mud",
        ),
        (
            6.0,
            GroundSurfaceKind::Rock,
            Color::srgb_u8(132, 136, 146),
            "SurfaceZone::Rock",
        ),
        (
            12.0,
            GroundSurfaceKind::Asphalt,
            Color::srgb_u8(62, 66, 74),
            "SurfaceZone::Asphalt",
        ),
    ];

    for (zone_x, kind, color, label) in zone_specs {
        let terrain_y = sample_hills_height(zone_x, zone_z, config);
        let center_y = terrain_y + zone_top_offset - zone_size.y * 0.5;
        commands.spawn((
            Name::new(label),
            Mesh3d(zone_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.92,
                ..default()
            })),
            Transform::from_xyz(zone_x, center_y, zone_z),
            RigidBody::Fixed,
            Collider::cuboid(zone_size.x * 0.5, zone_size.y * 0.5, zone_size.z * 0.5),
            CollisionGroups::new(GROUP_WORLD, Group::ALL),
            GroundSurfaceTag::new(kind),
        ));
    }
}

fn sample_hills_height(x: f32, z: f32, config: &PolygonConfig) -> f32 {
    let n = config.hills_noise_scale;
    let base = (x * n).sin() * 0.95 + (z * n * 1.17).cos() * 0.78;
    let ridge = ((x + z * 0.7) * n * 0.83).sin() * 0.55;
    let cross = ((x * 0.46) * n).cos() * ((z * 0.52) * n).sin() * 0.68;
    let radial = ((x * x + z * z).sqrt() * n * 0.31).sin() * 0.50;

    let mut height = (base + ridge + cross + radial) * config.hills_max_height * 0.42;

    let spawn_center = Vec2::new(0.0, 6.0);
    let to_spawn = Vec2::new(x - spawn_center.x, z - spawn_center.y);
    let distance = to_spawn.length();
    if distance < config.hills_spawn_flat_radius {
        let t = 1.0 - distance / config.hills_spawn_flat_radius;
        let smooth = t * t * (3.0 - 2.0 * t);
        let flatten = smooth * config.hills_spawn_flat_strength;
        height *= 1.0 - flatten;
    }

    height
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

fn create_checker_hills_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let green = [92, 132, 86, 255];
    let white = [236, 244, 234, 255];
    let checker_pixels = [
        green[0], green[1], green[2], green[3], white[0], white[1], white[2], white[3], white[0],
        white[1], white[2], white[3], green[0], green[1], green[2], green[3],
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
        perceptual_roughness: 0.94,
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
