use bevy::prelude::*;

use crate::ui::state::UiSettings;

use self::metrics::MetricsSnapshot;

pub mod metrics;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HudText;

pub fn spawn_hud_ui_system(mut commands: Commands) {
    commands
        .spawn((
            Name::new("HudRoot"),
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(14.0),
                left: px(14.0),
                width: px(380.0),
                padding: UiRect::all(px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.46)),
            BorderRadius::all(px(8.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                HudText,
                Text::new(""),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.94, 0.98)),
                Node {
                    width: percent(100.0),
                    ..default()
                },
            ));
        });
}

pub fn update_hud_text_system(
    settings: Res<UiSettings>,
    snapshot: Res<MetricsSnapshot>,
    mut text_q: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    let mut lines = Vec::new();
    if settings.show_movement_metrics {
        lines.push("Movement".to_owned());
        lines.extend(snapshot.movement_lines.clone());
    }
    if settings.show_fps_metrics {
        lines.push("".to_owned());
        lines.push("FPS".to_owned());
        lines.push(
            snapshot
                .fps_line
                .clone()
                .unwrap_or_else(|| "fps: no data".to_owned()),
        );
    }
    if settings.show_network_metrics {
        lines.push("".to_owned());
        lines.push("Network".to_owned());
        lines.extend(snapshot.network_lines.clone());
    }
    if settings.show_controls_hint {
        lines.push("".to_owned());
        lines.push("ESC: menu".to_owned());
    }

    *text = Text::new(lines.join("\n"));
}
