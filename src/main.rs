use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_rapier3d::plugin::NoUserData;
use bevy_rapier3d::plugin::PhysicsSet;
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use plugins::polygon::PolygonPlugin;
use systems::camera_move::*;
use systems::combat::*;
use systems::enemy_ai::*;
use systems::fire::*;
use systems::impact::*;
use systems::lock_cursor::*;
use systems::player_move::*;
use systems::player_respawn::*;
use systems::player_rotate::*;
use systems::setup::*;
use systems::shot_tracer::*;

use crate::resources::{combat_rules::CombatRules, input_settings::InputSettings};

mod components;
mod plugins;
mod resources;
mod systems;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            PolygonPlugin,
        ))
        .insert_resource(InputSettings {
            mouse_sensitivity: 0.0008,
        })
        .insert_resource(CombatRules::default())
        .add_message::<ImpactEvent>()
        .add_message::<DamageEvent>()
        .add_message::<DeathEvent>()
        .add_systems(Startup, (setup, lock_cursor_system))
        .add_systems(
            Update,
            (
                player_rotate_system,
                player_move_system,
                enemy_ai_state_system.run_if(on_timer(Duration::from_millis(120))),
                enemy_move_system.run_if(on_timer(Duration::from_millis(50))),
                fire_system,
                enemy_fire_system.run_if(on_timer(Duration::from_millis(50))),
                process_impact_system,
                debris_chip_lifetime_system,
                update_shot_tracer_system,
                apply_damage_system,
                schedule_player_respawn_on_death_system,
                handle_death_system,
                player_respawn_tick_system,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (camera_move_system.after(PhysicsSet::Writeback),),
        )
        .run();
}
