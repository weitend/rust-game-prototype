use bevy::prelude::*;
use bevy_rapier3d::plugin::NoUserData;
use bevy_rapier3d::plugin::PhysicsSet;
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use systems::bullet_hit::*;
use systems::bullet_lifetime::*;
use systems::camera_move::*;
use systems::fire::*;
use systems::impact_mark_lifetime::*;
use systems::lock_cursor::*;
use systems::player_move::*;
use systems::player_rotate::*;
use systems::setup::*;

use crate::resources::input_settings::InputSettings;

mod components;
mod resources;
mod systems;
mod utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RapierPhysicsPlugin::<NoUserData>::default()))
        .insert_resource(InputSettings {
            mouse_sensitivity: 0.0008,
        })
        .add_systems(Startup, (setup, lock_cursor_system))
        .add_systems(
            Update,
            (
                player_rotate_system,
                player_move_system,
                fire_system,
                bullet_lifetyme_system,
                impact_mark_lifetime_system,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                camera_move_system.after(PhysicsSet::Writeback),
                bullet_hit_system.after(PhysicsSet::Writeback),
            ),
        )
        .run();
}
