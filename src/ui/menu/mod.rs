use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::ui::state::{UiScreen, UiSettings};

#[derive(Component)]
pub struct PauseOverlayRoot;

#[derive(Component)]
pub struct PauseMenuPanel;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component, Clone, Copy)]
pub enum MenuButtonAction {
    Continue,
    Settings,
    Exit,
    Back,
}

#[derive(Component, Clone, Copy)]
pub enum SettingsToggleAction {
    Movement,
    Fps,
    Network,
    Hint,
}

#[derive(Component, Clone, Copy)]
pub struct SettingsToggleText {
    pub action: SettingsToggleAction,
}

pub fn spawn_pause_menu_ui_system(mut commands: Commands) {
    commands
        .spawn((
            Name::new("PauseOverlayRoot"),
            PauseOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("PauseMenuPanel"),
                    PauseMenuPanel,
                    Node {
                        width: px(360.0),
                        padding: UiRect::all(px(20.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12.0),
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.10, 0.13, 0.93)),
                    BorderRadius::all(px(10.0)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Paused"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.97, 1.0)),
                    ));
                    spawn_menu_button(panel, "Continue", MenuButtonAction::Continue);
                    spawn_menu_button(panel, "Settings", MenuButtonAction::Settings);
                    spawn_menu_button(panel, "Exit", MenuButtonAction::Exit);
                });

            parent
                .spawn((
                    Name::new("SettingsPanel"),
                    SettingsPanel,
                    Node {
                        width: px(420.0),
                        padding: UiRect::all(px(20.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10.0),
                        align_items: AlignItems::Stretch,
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.10, 0.13, 0.93)),
                    BorderRadius::all(px(10.0)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.97, 1.0)),
                    ));
                    spawn_toggle_button(panel, "Movement metrics", SettingsToggleAction::Movement);
                    spawn_toggle_button(panel, "FPS", SettingsToggleAction::Fps);
                    spawn_toggle_button(panel, "Network", SettingsToggleAction::Network);
                    spawn_toggle_button(panel, "Hint (ESC)", SettingsToggleAction::Hint);
                    spawn_menu_button(panel, "Back", MenuButtonAction::Back);
                });
        });
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands<'_>, label: &str, action: MenuButtonAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: percent(100.0),
                height: px(46.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.24, 0.30)),
            BorderRadius::all(px(8.0)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.97, 1.0)),
            ));
        });
}

fn spawn_toggle_button(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    action: SettingsToggleAction,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: percent(100.0),
                height: px(44.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(px(12.0), px(0.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.19, 0.22, 0.29)),
            BorderRadius::all(px(8.0)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.97, 1.0)),
            ));
            button.spawn((
                SettingsToggleText { action },
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.86, 0.98)),
            ));
        });
}

pub fn esc_toggle_ui_screen_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_screen: Res<State<UiScreen>>,
    mut next_ui_screen: ResMut<NextState<UiScreen>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    match ui_screen.get() {
        UiScreen::Hidden => next_ui_screen.set(UiScreen::PauseMenu),
        UiScreen::PauseMenu | UiScreen::Settings => next_ui_screen.set(UiScreen::Hidden),
    }
}

pub fn apply_cursor_mode_for_ui_screen_system(
    ui_screen: Res<State<UiScreen>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    match ui_screen.get() {
        UiScreen::Hidden => {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
        }
        UiScreen::PauseMenu | UiScreen::Settings => {
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}

pub fn sync_menu_visibility_system(
    ui_screen: Res<State<UiScreen>>,
    mut nodes_q: Query<(
        &mut Node,
        Has<PauseOverlayRoot>,
        Has<PauseMenuPanel>,
        Has<SettingsPanel>,
    )>,
) {
    let (overlay_display, pause_display, settings_display) = match ui_screen.get() {
        UiScreen::Hidden => (Display::None, Display::None, Display::None),
        UiScreen::PauseMenu => (Display::Flex, Display::Flex, Display::None),
        UiScreen::Settings => (Display::Flex, Display::None, Display::Flex),
    };

    for (mut node, is_overlay, is_pause, is_settings) in &mut nodes_q {
        if is_overlay {
            node.display = overlay_display;
        }
        if is_pause {
            node.display = pause_display;
        }
        if is_settings {
            node.display = settings_display;
        }
    }
}

pub fn menu_button_interaction_system(
    mut app_exit: MessageWriter<AppExit>,
    mut ui_screen: ResMut<NextState<UiScreen>>,
    mut q: Query<
        (&Interaction, &MenuButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, action, mut background_color) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb(0.34, 0.41, 0.53));
                match action {
                    MenuButtonAction::Continue => ui_screen.set(UiScreen::Hidden),
                    MenuButtonAction::Settings => ui_screen.set(UiScreen::Settings),
                    MenuButtonAction::Exit => {
                        app_exit.write(AppExit::Success);
                    }
                    MenuButtonAction::Back => ui_screen.set(UiScreen::PauseMenu),
                }
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb(0.27, 0.33, 0.43));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb(0.20, 0.24, 0.30));
            }
        }
    }
}

pub fn settings_toggle_interaction_system(
    mut settings: ResMut<UiSettings>,
    mut q: Query<
        (&Interaction, &SettingsToggleAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, action, mut background_color) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb(0.34, 0.41, 0.53));
                match action {
                    SettingsToggleAction::Movement => {
                        settings.show_movement_metrics = !settings.show_movement_metrics
                    }
                    SettingsToggleAction::Fps => {
                        settings.show_fps_metrics = !settings.show_fps_metrics
                    }
                    SettingsToggleAction::Network => {
                        settings.show_network_metrics = !settings.show_network_metrics
                    }
                    SettingsToggleAction::Hint => {
                        settings.show_controls_hint = !settings.show_controls_hint
                    }
                }
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb(0.27, 0.33, 0.43));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb(0.19, 0.22, 0.29));
            }
        }
    }
}

pub fn update_settings_toggle_labels_system(
    settings: Res<UiSettings>,
    mut labels: Query<(&SettingsToggleText, &mut Text)>,
) {
    for (toggle, mut text) in &mut labels {
        let enabled = match toggle.action {
            SettingsToggleAction::Movement => settings.show_movement_metrics,
            SettingsToggleAction::Fps => settings.show_fps_metrics,
            SettingsToggleAction::Network => settings.show_network_metrics,
            SettingsToggleAction::Hint => settings.show_controls_hint,
        };
        *text = Text::new(if enabled { "ON" } else { "OFF" });
    }
}
