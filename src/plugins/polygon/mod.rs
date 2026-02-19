mod config;
mod layout;
mod sections;
mod systems;

use bevy::prelude::*;

pub use config::PolygonConfig;
use systems::setup_polygon_system;

pub struct PolygonPlugin;

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PolygonConfig>()
            .add_systems(Startup, setup_polygon_system);
    }
}
