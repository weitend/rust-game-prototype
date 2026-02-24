use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        aim_marker::{AimMarker, ArtilleryVignette},
        combat::{Health, Team},
        enemy::{Enemy, EnemyAi, EnemyControllerState},
        fire_control::FireControl,
        follow_camera::FollowCamera,
        intent::EnemyIntent,
        shoot_origin::ShootOrigin,
        weapon::HitscanWeapon,
    },
    resources::{
        aim_settings::{AIM_MARKER_RENDER_LAYER, AimSettings},
        impact_assets::ImpactAssets,
        player_spawn::{PlayerRespawnState, PlayerTemplate},
        tank_settings::TankSettings,
        tracer_assets::TracerAssets,
    },
    systems::player_respawn::spawn_player_from_template,
    utils::{collision_groups::enemy_collision_groups, muzzle::compute_muzzle},
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    aim_settings: Res<AimSettings>,
    tank_settings: Res<TankSettings>,
) {
    let muzzle_padding: f32 = 0.015;

    // Player
    let player_hull_mesh = meshes.add(Cuboid::new(1.6, 0.74, 2.2));
    let player_turret_mesh = meshes.add(Cuboid::new(1.05, 0.34, 1.05));
    let player_barrel_mesh = meshes.add(Cuboid::new(0.18, 0.18, 1.26));
    let player_muzzle_offset = Vec3::new(0.0, 0.55, -1.70);

    let player_template = PlayerTemplate {
        mesh: player_hull_mesh,
        material: materials.add(Color::srgb_u8(10, 144, 255)),
        turret_mesh: player_turret_mesh,
        turret_material: materials.add(Color::srgb_u8(20, 167, 255)),
        barrel_mesh: player_barrel_mesh,
        barrel_material: materials.add(Color::srgb_u8(58, 78, 104)),
        muzzle_offset: player_muzzle_offset,
        collider_half_extents: Vec3::new(0.80, 0.37, 1.10),
        spawn_translation: Vec3::new(0.0, 0.9, 6.0),
        max_health: 100.0,
        shots_per_second: 5.0,
        weapon_damage: 25.0,
        weapon_range: 45.0,
    };
    spawn_player_from_template(&mut commands, &player_template, &tank_settings);
    commands.insert_resource(player_template);
    commands.insert_resource(PlayerRespawnState::default());

    // Enemies
    let enemy_rows = 1;
    let enemy_cols = 1;
    let enemy_spacing_x = 3.6;
    let enemy_spacing_z = 3.6;
    let enemy_origin = Vec3::new(-7.2, 0.9, -22.0);
    let enemy_sps = 2.0;
    let enemy_material = materials.add(Color::srgb_u8(208, 64, 64));
    let enemy_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let enemy_muzzle_offset = meshes
        .get(&enemy_mesh)
        .and_then(|mesh| compute_muzzle(mesh, muzzle_padding))
        .unwrap_or(Vec3::ZERO);

    for row in 0..enemy_rows {
        for col in 0..enemy_cols {
            let enemy_pos = enemy_origin
                + Vec3::new(
                    col as f32 * enemy_spacing_x,
                    0.0,
                    row as f32 * enemy_spacing_z,
                );

            commands.spawn((
                Mesh3d(enemy_mesh.clone()),
                MeshMaterial3d(enemy_material.clone()),
                Transform::from_translation(enemy_pos),
                Enemy,
                Team::Enemy,
                Health::new(100.0),
                EnemyAi::new(30.0, 16.0),
                EnemyControllerState::default(),
                EnemyIntent::default(),
                ShootOrigin {
                    muzzle_offset: enemy_muzzle_offset,
                },
                enemy_collision_groups(),
                Collider::cuboid(0.48, 0.5, 0.48),
                KinematicCharacterController {
                    offset: CharacterLength::Absolute(0.003),
                    slide: true,
                    apply_impulse_to_dynamic_bodies: false,
                    filter_flags: QueryFilterFlags::EXCLUDE_DYNAMIC
                        | QueryFilterFlags::EXCLUDE_SENSORS,
                    // Cheaper controller for crowds of enemies.
                    autostep: None,
                    snap_to_ground: None,
                    ..default()
                },
                FireControl {
                    cooldown: Timer::from_seconds(1.0 / enemy_sps, TimerMode::Repeating),
                },
                HitscanWeapon {
                    damage: 10.0,
                    range: 35.0,
                },
            ));
        }
    }

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FollowCamera,
        RenderLayers::from_layers(&[0, AIM_MARKER_RENDER_LAYER]),
    ));

    let aim_marker_mesh = meshes.add(Cylinder::new(
        aim_settings.marker_radius,
        aim_settings.marker_height,
    ));
    let aim_marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.91, 0.22, 0.25, 0.48),
        emissive: Color::srgb(0.55, 0.10, 0.12).into(),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Name::new("AimMarker"),
        Mesh3d(aim_marker_mesh),
        MeshMaterial3d(aim_marker_material),
        Transform::from_xyz(0.0, -1000.0, 0.0),
        Visibility::Hidden,
        AimMarker,
        RenderLayers::layer(AIM_MARKER_RENDER_LAYER),
    ));

    commands.spawn((
        Name::new("ArtilleryVignette"),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            border: UiRect::all(px(aim_settings.vignette_border_px)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ArtilleryVignette,
    ));

    // Impact mark assets startup
    let impact_radius = 0.06;
    let chip_size = 0.06;
    let chip_mesh = meshes.add(Cuboid::new(chip_size, chip_size, chip_size));
    let chip_fallback_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(74, 74, 74),
        perceptual_roughness: 0.92,
        ..default()
    });
    commands.insert_resource(ImpactAssets {
        radius: impact_radius,
        crater_size: 0.22,
        crater_depth: 0.13,
        min_marks_per_impact: 4,
        max_marks_per_impact: 10,
        damage_for_max_web: 45.0,
        base_web_radius: 0.06,
        max_web_radius: 0.24,
        max_marks_per_frame: 80,
        chip_mesh,
        chip_fallback_material,
        min_chips_per_impact: 2,
        max_chips_per_impact: 5,
        chip_size,
        chip_speed: 4.6,
        chip_lifetime_secs: 1.1,
        max_chips_per_frame: 36,
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
