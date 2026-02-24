mod config;
mod layout;
mod sections;
mod systems;
mod teleports;

use bevy::prelude::*;
use bevy_rapier3d::plugin::PhysicsSet;

pub use config::PolygonConfig;
use systems::setup_polygon_system;
use teleports::{sync_teleport_labels_system, teleport_player_system, TeleportRuntime};

pub struct PolygonPlugin;

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PolygonConfig>()
            .init_resource::<TeleportRuntime>()
            .add_systems(Startup, setup_polygon_system)
            .add_systems(
                PostUpdate,
                (
                    teleport_player_system
                        .after(PhysicsSet::Writeback)
                        .before(crate::systems::camera_move::camera_move_system),
                    sync_teleport_labels_system
                        .after(crate::systems::camera_move::camera_move_system),
                ),
            );
    }
}
