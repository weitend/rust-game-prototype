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
            GroundContact, TankBarrel, TankBarrelState, TankHull, TankMuzzle, TankParts,
            TankTurret, TankTurretState, TrackVisualPhase,
        },
        weapon::{HitscanWeapon, ProjectileWeaponProfile},
    },
    resources::{
        player_physics_settings::PlayerPhysicsSettings,
        player_spawn::{PlayerRespawnState, PlayerTemplate},
    },
    systems::combat::DeathEvent,
    utils::{
        collision_groups::player_collision_groups,
        tank_physics::{tank_additional_mass_properties, tank_collider, tank_suspension},
        tank_visual::{
            spawn_track_visuals_for_entity, BARREL_PIVOT_LOCAL_OFFSET, BARREL_VISUAL_LOCAL_OFFSET,
            MUZZLE_LOCAL_OFFSET, TURRET_LOCAL_OFFSET,
        },
    },
};

pub fn spawn_player_from_template(
    commands: &mut Commands,
    template: &PlayerTemplate,
    physics_settings: &PlayerPhysicsSettings,
) {
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
        TrackVisualPhase::default(),
        PlayerIntent::default(),
        GroundContact::default(),
    ));
    let player_entity_id = player_entity.id();
    player_entity.insert((
        ShootOrigin {
            muzzle_offset: template.muzzle_offset,
        },
        player_collision_groups(),
        tank_collider(template.collider_half_extents, physics_settings),
        tank_suspension(template.collider_half_extents),
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        FireControl {
            cooldown: Timer::from_seconds(1.0 / template.shots_per_second, TimerMode::Repeating),
        },
        HitscanWeapon {
            damage: template.weapon_damage,
            range: template.weapon_range,
        },
        ProjectileWeaponProfile::default(),
    ));
    player_entity.insert((
        RigidBody::Dynamic,
        Velocity::zero(),
        ExternalForce::default(),
        tank_additional_mass_properties(physics_settings),
        Sleeping::disabled(),
        Damping {
            linear_damping: physics_settings.linear_damping,
            angular_damping: physics_settings.angular_damping,
        },
    ));

    spawn_track_visuals_for_entity(commands, template, player_entity_id, Some(player_entity_id));

    let turret_entity = commands
        .spawn((
            Name::new("TankTurret"),
            Mesh3d(template.turret_mesh.clone()),
            MeshMaterial3d(template.turret_material.clone()),
            Transform::from_translation(TURRET_LOCAL_OFFSET),
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
            Transform::from_translation(BARREL_PIVOT_LOCAL_OFFSET),
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
            Transform::from_translation(BARREL_VISUAL_LOCAL_OFFSET),
        ))
        .id();

    let muzzle_entity = commands
        .spawn((
            Name::new("TankMuzzle"),
            Transform::from_translation(MUZZLE_LOCAL_OFFSET),
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

    spawn_player_from_template(&mut commands, &template, &physics_settings);
    respawn.pending = false;
}
