use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use self::{
    hud::{
        metrics::{
            FpsMetricsSnapshot, MetricsSnapshot, MovementMetricsSnapshot, NetworkMetricsSnapshot,
            collect_fps_metrics_system, collect_movement_metrics_system,
            collect_network_metrics_system, compose_metrics_snapshot_system,
        },
        spawn_hud_ui_system, update_hud_text_system,
    },
    menu::{
        apply_cursor_mode_for_ui_screen_system, esc_toggle_ui_screen_system,
        menu_button_interaction_system, settings_toggle_interaction_system,
        spawn_pause_menu_ui_system, sync_menu_visibility_system,
        update_settings_toggle_labels_system,
    },
    state::{UiScreen, UiSettings},
};

pub mod hud;
pub mod menu;
pub mod state;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<UiScreen>()
            .init_resource::<UiSettings>()
            .init_resource::<MovementMetricsSnapshot>()
            .init_resource::<FpsMetricsSnapshot>()
            .init_resource::<NetworkMetricsSnapshot>()
            .init_resource::<MetricsSnapshot>()
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, (spawn_hud_ui_system, spawn_pause_menu_ui_system))
            .add_systems(
                Update,
                (
                    esc_toggle_ui_screen_system,
                    apply_cursor_mode_for_ui_screen_system,
                    menu_button_interaction_system,
                    settings_toggle_interaction_system,
                    sync_menu_visibility_system,
                    update_settings_toggle_labels_system,
                ),
            )
            .add_systems(
                Update,
                (
                    collect_movement_metrics_system,
                    collect_fps_metrics_system,
                    collect_network_metrics_system,
                    compose_metrics_snapshot_system,
                )
                    .chain()
                    .run_if(on_timer(Duration::from_millis(200))),
            )
            .add_systems(
                Update,
                update_hud_text_system.after(compose_metrics_snapshot_system),
            );
    }
}
