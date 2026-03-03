use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::systems::track_visual::{track_link_count, track_loop_length_m, track_pose_from_phase};
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
            TankTurret, TankTurretState, TrackLinkVisual, TrackSide, TrackVisualPhase,
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
    },
};

pub fn spawn_player_from_template(
    commands: &mut Commands,
    template: &PlayerTemplate,
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

    let link_count = track_link_count();
    let loop_length_m = track_loop_length_m();
    let guide_inset_x = 0.02_f32;
    for guide_idx in 0..link_count {
        let phase = (guide_idx as f32 / link_count as f32) * loop_length_m;
        for side in [TrackSide::Left, TrackSide::Right] {
            let (mut guide_position, guide_rotation) = track_pose_from_phase(side, phase);
            guide_position.x += match side {
                TrackSide::Left => guide_inset_x,
                TrackSide::Right => -guide_inset_x,
            };
            let name = match side {
                TrackSide::Left => "TankTrack::LeftGuide",
                TrackSide::Right => "TankTrack::RightGuide",
            };
            let guide = commands
                .spawn((
                    Name::new(name),
                    Mesh3d(template.track_belt_mesh.clone()),
                    MeshMaterial3d(template.track_belt_material.clone()),
                    Transform::from_translation(guide_position).with_rotation(guide_rotation),
                ))
                .id();
            commands.entity(player_entity_id).add_child(guide);
        }
    }

    for link_idx in 0..link_count {
        let phase = (link_idx as f32 / link_count as f32) * loop_length_m;
        let (left_position, left_rotation) = track_pose_from_phase(TrackSide::Left, phase);
        let left_link = commands
            .spawn((
                Name::new("TankTrack::LeftLink"),
                Mesh3d(template.track_link_mesh.clone()),
                MeshMaterial3d(template.track_link_material.clone()),
                Transform::from_translation(left_position).with_rotation(left_rotation),
                OwnedBy {
                    entity: player_entity_id,
                },
                TrackLinkVisual {
                    side: TrackSide::Left,
                    base_phase_m: phase,
                },
            ))
            .id();

        let (right_position, right_rotation) = track_pose_from_phase(TrackSide::Right, phase);
        let right_link = commands
            .spawn((
                Name::new("TankTrack::RightLink"),
                Mesh3d(template.track_link_mesh.clone()),
                MeshMaterial3d(template.track_link_material.clone()),
                Transform::from_translation(right_position).with_rotation(right_rotation),
                OwnedBy {
                    entity: player_entity_id,
                },
                TrackLinkVisual {
                    side: TrackSide::Right,
                    base_phase_m: phase,
                },
            ))
            .id();

        commands.entity(player_entity_id).add_child(left_link);
        commands.entity(player_entity_id).add_child(right_link);
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
