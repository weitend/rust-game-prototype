mod config;
mod layout;
mod sections;
mod systems;
mod teleports;

use bevy::{
    asset::AssetApp, image::Image, mesh::Mesh, pbr::StandardMaterial, prelude::*,
};
use bevy_rapier3d::plugin::PhysicsSet;

use crate::resources::run_mode::{AppRunMode, RunMode};

pub use config::{PolygonConfig, PolygonMapMode};
use systems::{assign_obstacle_net_ids_system, setup_polygon_system};
use teleports::{TeleportRuntime, sync_teleport_labels_system, teleport_player_system};

pub enum PolygonPlugin {
    Training,
    Hills,
}

impl Default for PolygonPlugin {
    fn default() -> Self {
        Self::Training
    }
}

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        let map_mode = match self {
            PolygonPlugin::Training => PolygonMapMode::TrainingGround,
            PolygonPlugin::Hills => PolygonMapMode::HillsDemo,
        };
        let mut config = app
            .world()
            .get_resource::<PolygonConfig>()
            .cloned()
            .unwrap_or_default();
        config.map_mode = map_mode;

        if !app.world().contains_resource::<Assets<Image>>() {
            app.init_asset::<Image>();
        }
        if !app.world().contains_resource::<Assets<Mesh>>() {
            app.init_asset::<Mesh>();
        }
        if !app.world().contains_resource::<Assets<StandardMaterial>>() {
            app.init_asset::<StandardMaterial>();
        }

        app.insert_resource(config)
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
