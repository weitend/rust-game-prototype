use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_rapier3d::plugin::PhysicsSet;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Friction, Group, RigidBody};

use crate::{
    resources::run_mode::{AppRunMode, RunMode},
    systems::{
        aim_marker::{update_aim_marker_system, update_artillery_vignette_system},
        camera_move::camera_move_system,
        combat::{DamageEvent, DeathEvent, apply_damage_system, handle_death_system},
        enemy_ai::{enemy_ai_state_system, enemy_fire_system, enemy_move_system},
        fire::fire_system,
        impact::{
            ImpactEvent, debris_chip_lifetime_system, process_impact_system,
            route_impact_damage_system,
        },
        intent::{
            enemy_intent_from_ai_system, player_input_intent_system,
            resolve_local_player_context_system,
        },
        invariants::debug_validate_invariants_system,
        lock_cursor::lock_cursor_system,
        player_respawn::{player_respawn_tick_system, schedule_player_respawn_on_death_system},
        setup::setup,
        shot_tracer::update_shot_tracer_system,
        tank_aim::{tank_barrel_pitch_system, tank_turret_yaw_system},
        tank_move::tank_hull_move_system,
    },
    utils::collision_groups::GROUP_WORLD,
};

use super::polygon::{PolygonConfig, PolygonPlugin};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ImpactEvent>()
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_systems(Startup, setup_server_ground_system.run_if(is_server_mode))
            .add_systems(
                Update,
                (
                    resolve_local_player_context_system,
                    player_input_intent_system.run_if(is_client_like_mode),
                    tank_hull_move_system,
                    tank_turret_yaw_system,
                    tank_barrel_pitch_system,
                    enemy_ai_state_system.run_if(on_timer(Duration::from_millis(120))),
                    enemy_intent_from_ai_system,
                    enemy_move_system.run_if(on_timer(Duration::from_millis(50))),
                    fire_system,
                    enemy_fire_system
                        .run_if(on_timer(Duration::from_millis(50)))
                        .run_if(is_client_like_mode),
                    route_impact_damage_system.run_if(is_server_like_mode),
                    apply_damage_system.run_if(is_server_like_mode),
                    schedule_player_respawn_on_death_system.run_if(is_client_like_mode),
                    handle_death_system.run_if(is_server_like_mode),
                    player_respawn_tick_system.run_if(is_client_like_mode),
                )
                    .chain(),
            );
    }
}

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PolygonPlugin)
            .add_systems(Startup, (setup, lock_cursor_system))
            .add_systems(
                Update,
                (
                    update_aim_marker_system,
                    update_artillery_vignette_system,
                    process_impact_system,
                    debris_chip_lifetime_system,
                    update_shot_tracer_system,
                ),
            )
            .add_systems(
                Update,
                debug_validate_invariants_system.run_if(on_timer(Duration::from_secs(3))),
            )
            .add_systems(
                PostUpdate,
                (camera_move_system.after(PhysicsSet::Writeback),),
            );
    }
}

fn is_client_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Client | RunMode::Host)
}

fn is_server_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Server | RunMode::Host)
}

fn is_server_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Server)
}

fn setup_server_ground_system(mut commands: Commands) {
    let config = PolygonConfig::default().sanitized();
    let size = config.platform_size();

    commands.spawn((
        Name::new("ServerGround"),
        Transform::from_xyz(0.0, -0.5 * size.y, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(0.5 * size.x, 0.5 * size.y, 0.5 * size.z),
        CollisionGroups::new(GROUP_WORLD, Group::ALL),
        Friction::coefficient(0.0),
    ));
}
