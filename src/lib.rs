use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::transform::TransformPlugin;
use bevy_rapier3d::plugin::NoUserData;
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use plugins::multiplayer::MultiplayerPlugin;
use plugins::polygon::PolygonPlugin;
use plugins::runtime::{PresentationPlugin, SimulationPlugin};
use ui::UiPlugin;

use crate::resources::{
    aim_settings::AimSettings, combat_rules::CombatRules,
    ground_surface_catalog::GroundSurfaceCatalog,
    ground_surface_visual_catalog::{GroundSurfaceVisualCatalog, GroundVisualSeason},
    local_player::LocalPlayerContext,
    player_physics_settings::PlayerPhysicsSettings, run_mode::AppRunMode,
    tank_settings::TankSettings,
};

pub mod components;
pub mod network;
pub mod plugins;
pub mod resources;
pub mod systems;
pub mod ui;
pub mod utils;

pub use resources::run_mode::RunMode;

pub fn run_app(run_mode: RunMode) {
    eprintln!("Run mode: {}", run_mode.as_str());

    let mut app = App::new();
    app.insert_resource(AppRunMode(run_mode))
        .insert_resource(AimSettings::default())
        .insert_resource(LocalPlayerContext::default())
        .insert_resource(PlayerPhysicsSettings::default())
        .insert_resource(GroundSurfaceCatalog::default())
        .insert_resource(GroundSurfaceVisualCatalog::with_season(
            GroundVisualSeason::Temperate,
        ))
        .insert_resource(TankSettings::default())
        .insert_resource(CombatRules::default());

    match run_mode {
        RunMode::Server => {
            app.add_plugins((
                MinimalPlugins,
                AssetPlugin::default(),
                TransformPlugin,
                RapierPhysicsPlugin::<NoUserData>::default(),
            ));
        }
        RunMode::Client | RunMode::Host => {
            app.add_plugins((DefaultPlugins, RapierPhysicsPlugin::<NoUserData>::default()));
        }
    }

    app.add_plugins(SimulationPlugin);
    app.add_plugins(MultiplayerPlugin);
    app.add_plugins(PolygonPlugin::Hills);
    if matches!(run_mode, RunMode::Client | RunMode::Host) {
        app.add_plugins(PresentationPlugin);
        app.add_plugins(UiPlugin);
    }

    app.run();
}
