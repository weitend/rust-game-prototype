use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        fire_control::FireControl,
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player, PlayerControllerState},
        shoot_origin::ShootOrigin,
        tank::{
            TankBarrel, TankBarrelState, TankHull, TankMuzzle, TankParts, TankTurret,
            TankTurretState,
        },
        weapon::{HitscanWeapon, ProjectileWeaponProfile},
    },
    resources::{
        player_motion_settings::PlayerMotionSettings,
        player_physics_settings::{PlayerHullPhysicsMode, PlayerPhysicsSettings},
        player_spawn::{PlayerRespawnState, PlayerTemplate},
    },
    systems::combat::DeathEvent,
    utils::collision_groups::player_collision_groups,
};

pub fn spawn_player_from_template(
    commands: &mut Commands,
    template: &PlayerTemplate,
    motion_settings: &PlayerMotionSettings,
    physics_settings: &PlayerPhysicsSettings,
) {
    let turret_local_offset = Vec3::new(0.0, 0.46, 0.0);
    let barrel_pivot_local_offset = Vec3::new(0.0, 0.09, -0.44);
    let barrel_visual_local_offset = Vec3::new(0.0, 0.0, -0.63);
    let muzzle_local_offset = Vec3::new(0.0, 0.0, -1.26);

    let mut player_entity = commands.spawn((
        Name::new("PlayerTank"),
        Mesh3d(template.mesh.clone()),
        MeshMaterial3d(template.material.clone()),
        Transform::from_translation(template.spawn_translation),
        Player,
        LocalPlayer,
        TankHull,
        Team::Player,
        Health::new(template.max_health),
        PlayerControllerState::default(),
        PlayerIntent::default(),
    ));
    let player_entity_id = player_entity.id();

    player_entity.insert((
        ShootOrigin {
            muzzle_offset: template.muzzle_offset,
        },
        player_collision_groups(),
        Collider::cuboid(
            template.collider_half_extents.x,
            template.collider_half_extents.y,
            template.collider_half_extents.z,
        ),
        FireControl {
            cooldown: Timer::from_seconds(1.0 / template.shots_per_second, TimerMode::Repeating),
        },
        HitscanWeapon {
            damage: template.weapon_damage,
            range: template.weapon_range,
        },
        ProjectileWeaponProfile::default(),
    ));
    match physics_settings.mode {
        PlayerHullPhysicsMode::KinematicController => {
            player_entity.insert(default_tank_controller(motion_settings));
        }
        PlayerHullPhysicsMode::DynamicForces => {
            player_entity.insert((
                RigidBody::Dynamic,
                Velocity::zero(),
                ExternalForce::default(),
                Damping {
                    linear_damping: physics_settings.dynamic_linear_damping,
                    angular_damping: physics_settings.dynamic_angular_damping,
                },
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            ));
        }
    }

    let turret_entity = commands
        .spawn((
            Name::new("TankTurret"),
            Mesh3d(template.turret_mesh.clone()),
            MeshMaterial3d(template.turret_material.clone()),
            Transform::from_translation(turret_local_offset),
            OwnedBy {
                entity: player_entity_id,
            },
            TankTurret,
            TankTurretState::default(),
        ))
        .id();

    let barrel_entity = commands
        .spawn((
            Name::new("TankBarrelPivot"),
            Transform::from_translation(barrel_pivot_local_offset),
            Visibility::default(),
            OwnedBy {
                entity: player_entity_id,
            },
            TankBarrel,
            TankBarrelState::default(),
        ))
        .id();

    let barrel_visual_entity = commands
        .spawn((
            Name::new("TankBarrel"),
            Mesh3d(template.barrel_mesh.clone()),
            MeshMaterial3d(template.barrel_material.clone()),
            Transform::from_translation(barrel_visual_local_offset),
        ))
        .id();

    let muzzle_entity = commands
        .spawn((
            Name::new("TankMuzzle"),
            Transform::from_translation(muzzle_local_offset),
            Visibility::default(),
            OwnedBy {
                entity: player_entity_id,
            },
            TankMuzzle,
        ))
        .id();

    commands
        .entity(barrel_entity)
        .add_child(barrel_visual_entity);
    commands.entity(barrel_entity).add_child(muzzle_entity);
    commands.entity(turret_entity).add_child(barrel_entity);
    commands.entity(player_entity_id).add_child(turret_entity);
    commands.entity(player_entity_id).insert(TankParts {
        turret: turret_entity,
        barrel: barrel_entity,
        muzzle: muzzle_entity,
    });
}

pub fn schedule_player_respawn_on_death_system(
    mut death_events: MessageReader<DeathEvent>,
    player_query: Query<(), With<Player>>,
    mut respawn: ResMut<PlayerRespawnState>,
) {
    for event in death_events.read() {
        if player_query.get(event.victim).is_ok() {
            respawn.pending = true;
            respawn.timer = Timer::from_seconds(respawn.delay_secs, TimerMode::Once);
            break;
        }
    }
}

pub fn player_respawn_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut respawn: ResMut<PlayerRespawnState>,
    template: Res<PlayerTemplate>,
    motion_settings: Res<PlayerMotionSettings>,
    physics_settings: Res<PlayerPhysicsSettings>,
    player_query: Query<(), With<Player>>,
) {
    if player_query.single().is_ok() {
        respawn.pending = false;
        return;
    }

    if !respawn.pending {
        return;
    }

    respawn.timer.tick(time.delta());
    if !respawn.timer.is_finished() {
        return;
    }

    spawn_player_from_template(
        &mut commands,
        &template,
        &motion_settings,
        &physics_settings,
    );
    respawn.pending = false;
}

fn default_tank_controller(settings: &PlayerMotionSettings) -> KinematicCharacterController {
    KinematicCharacterController {
        offset: CharacterLength::Absolute(settings.controller_offset),
        slide: true,
        apply_impulse_to_dynamic_bodies: false,
        filter_flags: QueryFilterFlags::EXCLUDE_DYNAMIC | QueryFilterFlags::EXCLUDE_SENSORS,
        autostep: Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(settings.autostep_height),
            min_width: CharacterLength::Absolute(settings.autostep_min_width),
            include_dynamic_bodies: false,
        }),
        snap_to_ground: Some(CharacterLength::Absolute(settings.snap_to_ground)),
        ..default()
    }
}
