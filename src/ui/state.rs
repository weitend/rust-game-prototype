use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UiScreen {
    #[default]
    Hidden,
    PauseMenu,
    Settings,
}

#[derive(Resource, Clone, Debug)]
pub struct UiSettings {
    pub show_movement_metrics: bool,
    pub show_fps_metrics: bool,
    pub show_network_metrics: bool,
    pub show_controls_hint: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_movement_metrics: true,
            show_fps_metrics: true,
            show_network_metrics: true,
            show_controls_hint: false,
        }
    }
}
