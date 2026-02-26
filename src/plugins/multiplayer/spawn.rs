use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    CharacterAutostep, CharacterLength, Collider, Damping, ExternalForce,
    KinematicCharacterController, LockedAxes, RigidBody, Velocity,
};

use crate::{
    components::{
        combat::{Health, Team},
        fire_control::FireControl,
        intent::PlayerIntent,
        owner::OwnedBy,
        player::{LocalPlayer, Player, PlayerControllerState},
        tank::{
            TankBarrel, TankBarrelState, TankHull, TankMuzzle, TankParts, TankTurret,
            TankTurretState,
        },
        weapon::{HitscanWeapon, ProjectileWeaponProfile},
    },
    resources::{
        player_motion_settings::PlayerMotionSettings,
        player_physics_settings::{PlayerHullPhysicsMode, PlayerPhysicsSettings},
        run_mode::{AppRunMode, RunMode},
    },
    utils::collision_groups::player_collision_groups,
};

use super::state::{NetworkControlledPlayer, ServerNetState};

pub(super) fn assign_player_entity_for_session(
    commands: &mut Commands,
    run_mode: &AppRunMode,
    preferred_host_local: Option<Entity>,
    motion_settings: &PlayerMotionSettings,
    physics_settings: &PlayerPhysicsSettings,
    session_id: u64,
) -> Entity {
    if matches!(run_mode.0, RunMode::Host) {
        if let Some(local_player_entity) = preferred_host_local {
            eprintln!(
                "[net-server] host bind: session_id={} entity={:?}",
                session_id, local_player_entity
            );
            return local_player_entity;
        }
    }
    spawn_network_controlled_player(commands, motion_settings, physics_settings, session_id)
}

pub(super) fn first_unbound_local_player_entity(
    state: &ServerNetState,
    local_player_q: &Query<Entity, (With<Player>, With<LocalPlayer>)>,
) -> Option<Entity> {
    local_player_q.iter().find(|candidate| {
        !state
            .sessions
            .values()
            .any(|s| s.player_entity == Some(*candidate))
    })
}

pub(super) fn find_unbound_local_player(
    local_player_q: &Query<Entity, (With<Player>, With<LocalPlayer>)>,
    reserved: &HashSet<Entity>,
) -> Option<Entity> {
    local_player_q
        .iter()
        .find(|candidate| !reserved.contains(candidate))
}

pub(super) fn spawn_network_controlled_player(
    commands: &mut Commands,
    motion_settings: &PlayerMotionSettings,
    physics_settings: &PlayerPhysicsSettings,
    session_id: u64,
) -> Entity {
    const TURRET_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.46, 0.0);
    const BARREL_PIVOT_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.09, -0.44);
    const MUZZLE_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.0, -1.26);

    let spawn_x = ((session_id.saturating_sub(1)) as f32) * 3.5;
    let mut entity = commands.spawn((
        Name::new(format!("NetPlayer#{session_id}")),
        Transform::from_translation(Vec3::new(spawn_x, 0.9, 6.0)),
        Player,
        TankHull,
        Team::Player,
        Health::new(100.0),
        PlayerControllerState::default(),
        PlayerIntent::default(),
        FireControl {
            cooldown: Timer::from_seconds(1.0 / 5.0, TimerMode::Repeating),
        },
        HitscanWeapon {
            damage: 25.0,
            range: 45.0,
        },
        ProjectileWeaponProfile::default(),
        NetworkControlledPlayer { session_id },
        player_collision_groups(),
        Collider::cuboid(0.80, 0.37, 1.10),
    ));

    match physics_settings.mode {
        PlayerHullPhysicsMode::KinematicController => {
            entity.insert(default_tank_controller(motion_settings));
        }
        PlayerHullPhysicsMode::DynamicForces => {
            entity.insert((
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

    let id = entity.id();
    let turret = commands
        .spawn((
            Name::new(format!("NetPlayer#{session_id}::Turret")),
            Transform::from_translation(TURRET_LOCAL_OFFSET),
            OwnedBy { entity: id },
            TankTurret,
            TankTurretState::default(),
        ))
        .id();
    let barrel = commands
        .spawn((
            Name::new(format!("NetPlayer#{session_id}::BarrelPivot")),
            Transform::from_translation(BARREL_PIVOT_LOCAL_OFFSET),
            Visibility::default(),
            OwnedBy { entity: id },
            TankBarrel,
            TankBarrelState::default(),
        ))
        .id();
    let muzzle = commands
        .spawn((
            Name::new(format!("NetPlayer#{session_id}::Muzzle")),
            Transform::from_translation(MUZZLE_LOCAL_OFFSET),
            Visibility::default(),
            OwnedBy { entity: id },
            TankMuzzle,
        ))
        .id();
    commands.entity(barrel).add_child(muzzle);
    commands.entity(turret).add_child(barrel);
    commands.entity(id).add_child(turret);
    commands.entity(id).insert(TankParts {
        turret,
        barrel,
        muzzle,
    });

    eprintln!(
        "[net-server] spawned network player: session_id={} entity={:?}",
        session_id, id
    );
    id
}

pub(super) fn default_tank_controller(settings: &PlayerMotionSettings) -> KinematicCharacterController {
    KinematicCharacterController {
        offset: CharacterLength::Absolute(settings.controller_offset),
        slide: true,
        apply_impulse_to_dynamic_bodies: false,
        filter_flags: bevy_rapier3d::prelude::QueryFilterFlags::EXCLUDE_DYNAMIC
            | bevy_rapier3d::prelude::QueryFilterFlags::EXCLUDE_SENSORS,
        autostep: Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(settings.autostep_height),
            min_width: CharacterLength::Absolute(settings.autostep_min_width),
            include_dynamic_bodies: false,
        }),
        snap_to_ground: Some(CharacterLength::Absolute(settings.snap_to_ground)),
        ..default()
    }
}

pub(super) fn despawn_network_owned_player(
    commands: &mut Commands,
    network_player_q: &Query<(), With<NetworkControlledPlayer>>,
    player_entity: Option<Entity>,
) {
    let Some(player_entity) = player_entity else {
        return;
    };
    if network_player_q.get(player_entity).is_ok() {
        commands.entity(player_entity).despawn();
    }
}
