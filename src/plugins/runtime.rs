use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_rapier3d::plugin::PhysicsSet;

use crate::{
    resources::run_mode::{AppRunMode, RunMode},
    systems::{
        aim_marker::{update_aim_marker_system, update_artillery_vignette_system},
        camera_move::camera_move_system,
        combat::{DamageEvent, DeathEvent, apply_damage_system, handle_death_system},
        fire::fire_system,
        impact::{
            ImpactEvent, debris_chip_lifetime_system, process_impact_system,
            route_impact_damage_system,
        },
        intent::{player_input_intent_system, resolve_local_player_context_system},
        invariants::debug_validate_invariants_system,
        lock_cursor::lock_cursor_system,
        player_respawn::{player_respawn_tick_system, schedule_player_respawn_on_death_system},
        projectile::projectile_step_system,
        setup::setup,
        shot_tracer::{
            spawn_hit_explosion_system, update_explosion_vfx_system, update_shot_tracer_system,
            update_smoke_puff_system,
        },
        tank_aim::{tank_barrel_pitch_system, tank_turret_yaw_system},
        tank_move::tank_hull_move_system,
        track_visual::{animate_track_visuals_system, integrate_track_visual_phase_fixed_system},
    },
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ImpactEvent>()
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_systems(
                FixedUpdate,
                (
                    tank_hull_move_system.before(PhysicsSet::StepSimulation),
                    integrate_track_visual_phase_fixed_system.after(tank_hull_move_system),
                ),
            )
            .add_systems(
                Update,
                (
                    resolve_local_player_context_system,
                    player_input_intent_system.run_if(is_client_like_mode),
                    tank_turret_yaw_system,
                    tank_barrel_pitch_system,
                    fire_system,
                    projectile_step_system.run_if(is_server_like_mode),
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
        app.add_systems(Startup, (setup, lock_cursor_system))
            .add_systems(
                Update,
                (
                    update_aim_marker_system
                        .after(tank_hull_move_system)
                        .after(tank_turret_yaw_system)
                        .after(tank_barrel_pitch_system)
                        .after(fire_system),
                    update_artillery_vignette_system,
                    process_impact_system,
                    debris_chip_lifetime_system,
                    spawn_hit_explosion_system,
                    update_shot_tracer_system,
                    update_smoke_puff_system,
                    update_explosion_vfx_system,
                ),
            )
            .add_systems(
                Update,
                debug_validate_invariants_system.run_if(on_timer(Duration::from_secs(3))),
            )
            .add_systems(
                PostUpdate,
                (
                    animate_track_visuals_system.after(PhysicsSet::Writeback),
                    camera_move_system.after(PhysicsSet::Writeback),
                ),
            );
    }
}

fn is_client_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Client | RunMode::Host)
}

fn is_server_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Server | RunMode::Host)
}
