use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        fire_control::FireControl,
        intent::PlayerIntent,
        player::{LocalPlayer, Player, PlayerControllerState},
        shoot_origin::ShootOrigin,
        tank::{TankBarrel, TankBarrelState, TankHull, TankMuzzle, TankTurret, TankTurretState},
        weapon::HitscanWeapon,
    },
    resources::{
        player_spawn::{PlayerRespawnState, PlayerTemplate},
        tank_settings::TankSettings,
    },
    systems::combat::DeathEvent,
    utils::collision_groups::player_collision_groups,
};

pub fn spawn_player_from_template(
    commands: &mut Commands,
    template: &PlayerTemplate,
    tank_settings: &TankSettings,
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
        default_tank_controller(tank_settings),
        FireControl {
            cooldown: Timer::from_seconds(1.0 / template.shots_per_second, TimerMode::Repeating),
        },
        HitscanWeapon {
            damage: template.weapon_damage,
            range: template.weapon_range,
        },
    ));

    player_entity.with_children(|parent| {
        parent
            .spawn((
                Name::new("TankTurret"),
                Mesh3d(template.turret_mesh.clone()),
                MeshMaterial3d(template.turret_material.clone()),
                Transform::from_translation(turret_local_offset),
                TankTurret,
                TankTurretState::default(),
            ))
            .with_children(|turret| {
                turret
                    .spawn((
                        Name::new("TankBarrelPivot"),
                        Transform::from_translation(barrel_pivot_local_offset),
                        Visibility::default(),
                        TankBarrel,
                        TankBarrelState::default(),
                    ))
                    .with_children(|barrel| {
                        barrel.spawn((
                            Name::new("TankBarrel"),
                            Mesh3d(template.barrel_mesh.clone()),
                            MeshMaterial3d(template.barrel_material.clone()),
                            Transform::from_translation(barrel_visual_local_offset),
                        ));
                        barrel.spawn((
                            Name::new("TankMuzzle"),
                            Transform::from_translation(muzzle_local_offset),
                            Visibility::default(),
                            TankMuzzle,
                        ));
                    });
            });
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
    tank_settings: Res<TankSettings>,
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

    spawn_player_from_template(&mut commands, &template, &tank_settings);
    respawn.pending = false;
}

fn default_tank_controller(settings: &TankSettings) -> KinematicCharacterController {
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
