use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::components::{
    intent::PlayerIntent,
    player::{LocalPlayer, Player},
};
use crate::resources::local_player::LocalPlayerContext;
use crate::ui::state::UiScreen;
use crate::utils::local_player::resolve_local_player_entity;

pub fn resolve_local_player_context_system(
    mut local_player_ctx: ResMut<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
) {
    let mut local_players = local_player_q.iter();
    local_player_ctx.entity = match (local_players.next(), local_players.next()) {
        (Some(entity), None) => Some(entity),
        _ => None,
    };
}

pub fn player_input_intent_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    ui_screen: Res<State<UiScreen>>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut player_q: Query<&mut PlayerIntent, With<Player>>,
) {
    let mut look_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        look_delta += event.delta;
    }

    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok(mut intent) = player_q.get_mut(player_entity) else {
        return;
    };

    if !matches!(ui_screen.get(), UiScreen::Hidden) {
        intent.throttle = 0.0;
        intent.turn = 0.0;
        intent.turret_yaw_delta = 0.0;
        intent.barrel_pitch_delta = 0.0;
        intent.fire_pressed = false;
        intent.fire_just_pressed = false;
        intent.artillery_active = false;
        return;
    }

    intent.throttle = axis_pressed(&keyboard, KeyCode::KeyW, KeyCode::KeyS).clamp(-1.0, 1.0);
    intent.turn = axis_pressed(&keyboard, KeyCode::KeyA, KeyCode::KeyD).clamp(-1.0, 1.0);
    intent.turret_yaw_delta = look_delta.x;
    intent.barrel_pitch_delta = look_delta.y;
    intent.fire_pressed = mouse_buttons.pressed(MouseButton::Left);
    intent.fire_just_pressed = mouse_buttons.just_pressed(MouseButton::Left);
    intent.artillery_active = mouse_buttons.pressed(MouseButton::Right);
}

fn axis_pressed(input: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    let pos = input.pressed(positive) as u8 as f32;
    let neg = input.pressed(negative) as u8 as f32;
    pos - neg
}
