use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        fire_control::FireControl,
        follow_camera::FollowCamera,
        player::{Player, PlayerControllerState},
        shoot_origin::ShootOrigin,
        weapon::HitscanWeapon,
    },
    resources::{impact_assets::ImpactAssets, tracer_assets::TracerAssets},
    utils::{collision_groups::player_collision_groups, muzzle::compute_muzzle},
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let muzzle_padding: f32 = 0.015;

    // Player
    let cube = (1.0, 1.0, 1.0);
    let mesh_handle = meshes.add(Cuboid::new(cube.0, cube.1, cube.2));
    let mesh = meshes.get(&mesh_handle);

    let muzzle_offset: Vec3 = mesh
        .and_then(|m| compute_muzzle(m, muzzle_padding))
        .unwrap_or(Vec3::ZERO);

    let sps: f32 = 5.0;

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb_u8(10, 144, 255))),
        Transform::from_xyz(0.0, 0.9, 6.0),
        Player,
        Team::Player,
        Health::new(100.0),
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
        },
        HitscanWeapon {
            damage: 25.0,
            range: 45.0,
        },
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FollowCamera,
    ));

    // Impact mark assets startup
    let impact_radius = 0.06;
    commands.insert_resource(ImpactAssets {
        radius: impact_radius,
        mesh: meshes.add(Sphere::new(impact_radius)),
        material: materials.add(Color::srgb_u8(30, 30, 30)),
        lifetime_secs: 30.0,
    });

    commands.insert_resource(TracerAssets {
        mesh: meshes.add(Sphere::new(0.03)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(255, 240, 120),
            unlit: true,
            ..default()
        }),
        speed: 65.0,
    });
}
