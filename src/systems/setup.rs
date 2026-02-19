use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        fire_control::FireControl,
        follow_camera::FollowCamera,
        obstacle::Obstacle,
        player::{Player, PlayerControllerState},
        shoot_origin::ShootOrigin,
    },
    resources::bullet_assets::BulletAssets,
    utils::{collision_groups::player_collision_groups, muzzle::compute_muzzle},
};

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bullet_radius: f32 = 0.015;

    // Platform
    let platform_size = (80.0, 0.2, 80.0);
    let tile_size = 3.0_f32;
    let uv_tiles = Vec2::new(platform_size.0 / tile_size, platform_size.2 / tile_size);

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
    let platform_material = materials.add(StandardMaterial {
        base_color_texture: Some(checker_texture),
        uv_transform: Affine2::from_scale(uv_tiles),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            platform_size.0,
            platform_size.1,
            platform_size.2,
        ))),
        MeshMaterial3d(platform_material),
        Transform::from_xyz(0.0, -0.1, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(
            platform_size.0 * 0.5,
            platform_size.1 * 0.5,
            platform_size.2 * 0.5,
        ),
        Friction::coefficient(0.0),
    ));

    // Obstacle
    let obstacle_material = materials.add(Color::srgb_u8(235, 120, 170));
    let obstacle_specs = [
        (Vec3::new(0.0, 1.2, -10.0), Vec3::new(8.0, 2.4, 1.2)),
        (Vec3::new(-9.0, 1.0, -4.0), Vec3::new(2.0, 2.0, 2.0)),
        (Vec3::new(8.0, 1.5, -6.5), Vec3::new(1.5, 3.0, 1.5)),
        (Vec3::new(3.5, 0.75, -2.5), Vec3::new(2.5, 1.5, 2.5)),
        (Vec3::new(-3.5, 0.75, -2.5), Vec3::new(2.5, 1.5, 2.5)),
        (Vec3::new(0.0, 2.5, -18.0), Vec3::new(3.0, 5.0, 3.0)),
    ];

    for (pos, size) in obstacle_specs {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(obstacle_material.clone()),
            Transform::from_translation(pos),
            Obstacle,
            RigidBody::Fixed,
            Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
            ActiveEvents::COLLISION_EVENTS,
        ));
    }

    // Player
    let cube = (1.0, 1.0, 1.0);
    let mesh_handle = meshes.add(Cuboid::new(cube.0, cube.1, cube.2));
    let mesh = meshes.get(&mesh_handle);

    let muzzle_offset: Vec3 = mesh
        .and_then(|m| compute_muzzle(m, bullet_radius))
        .unwrap_or(Vec3::ZERO);

    let sps: f32 = 5.0;

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb_u8(10, 144, 255))),
        Transform::from_xyz(0.0, 0.9, 0.0),
        Player,
        PlayerControllerState::default(),
        ShootOrigin { muzzle_offset },
        player_collision_groups(),
        Collider::cuboid(0.48, 0.5, 0.48),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.003),
            slide: true,
            apply_impulse_to_dynamic_bodies: false,
            filter_flags: QueryFilterFlags::EXCLUDE_DYNAMIC | QueryFilterFlags::EXCLUDE_SENSORS,
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.25),
                min_width: CharacterLength::Absolute(0.2),
                include_dynamic_bodies: false,
            }),
            snap_to_ground: Some(CharacterLength::Absolute(0.03)),
            ..default()
        },
        FireControl {
            cooldown: Timer::from_seconds(1.0 / sps, TimerMode::Repeating),
            shots_per_second: sps,
        },
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, -4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FollowCamera,
    ));

    // Bullet config startup
    let impact_radius = 0.06;
    commands.insert_resource(BulletAssets {
        radius: bullet_radius,
        speed: 2.0,
        bullet_lifetime_secs: 3.0,
        mesh: meshes.add(Sphere::new(bullet_radius)),
        material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
        impact_radius,
        impact_mesh: meshes.add(Sphere::new(impact_radius)),
        impact_material: materials.add(Color::srgb_u8(30, 30, 30)),
    })
}
