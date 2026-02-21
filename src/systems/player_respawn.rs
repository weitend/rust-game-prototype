use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        fire_control::FireControl,
        player::{LocalPlayer, Player, PlayerControllerState},
        shoot_origin::ShootOrigin,
        tank::{TankBarrel, TankHull, TankMuzzle, TankTurret},
        weapon::HitscanWeapon,
    },
    resources::player_spawn::{PlayerRespawnState, PlayerTemplate},
    systems::combat::DeathEvent,
    utils::collision_groups::player_collision_groups,
};

pub fn spawn_player_from_template(commands: &mut Commands, template: &PlayerTemplate) {
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
        default_player_controller(),
        FireControl {
            cooldown: Timer::from_seconds(1.0 / template.shots_per_second, TimerMode::Repeating),
        },
        HitscanWeapon {
            damage: template.weapon_damage,
            range: template.weapon_range,
        },
    ));

    player_entity
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("TankTurret"),
                    Mesh3d(template.turret_mesh.clone()),
                    MeshMaterial3d(template.turret_material.clone()),
                    Transform::from_translation(turret_local_offset),
                    TankTurret,
                ))
                .with_children(|turret| {
                    turret
                        .spawn((
                            Name::new("TankBarrelPivot"),
                            Transform::from_translation(barrel_pivot_local_offset),
                            Visibility::default(),
                            TankBarrel,
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

    spawn_player_from_template(&mut commands, &template);
    respawn.pending = false;
}

fn default_player_controller() -> KinematicCharacterController {
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
    }
}
