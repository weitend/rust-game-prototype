mod config;
mod layout;
mod sections;
mod systems;
mod teleports;

use bevy::{image::Image, mesh::Mesh, pbr::StandardMaterial, prelude::*};
use bevy_rapier3d::plugin::PhysicsSet;

use crate::resources::run_mode::{AppRunMode, RunMode};

pub use config::PolygonConfig;
use systems::{assign_obstacle_net_ids_system, setup_polygon_system};
use teleports::{TeleportRuntime, sync_teleport_labels_system, teleport_player_system};

pub struct PolygonPlugin;

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PolygonConfig>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<TeleportRuntime>()
            .add_systems(Startup, setup_polygon_system)
            .add_systems(
                Startup,
                assign_obstacle_net_ids_system.after(setup_polygon_system),
            )
            .add_systems(
                PostUpdate,
                (
                    teleport_player_system
                        .run_if(is_client_like_mode)
                        .after(PhysicsSet::Writeback)
                        .before(crate::systems::camera_move::camera_move_system),
                    sync_teleport_labels_system
                        .run_if(is_client_like_mode)
                        .after(crate::systems::camera_move::camera_move_system),
                ),
            );
    }
}

fn is_client_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Client | RunMode::Host)
}
