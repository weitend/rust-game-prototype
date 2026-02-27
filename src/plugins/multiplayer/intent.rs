use bevy::prelude::*;

use crate::{
    components::{
        intent::PlayerIntent,
        player::{LocalPlayer, Player},
    },
    network::protocol::ClientInput,
};

pub(super) fn player_intent_from_client_input(input: &ClientInput) -> PlayerIntent {
    PlayerIntent {
        throttle: input.throttle.clamp(-1.0, 1.0),
        turn: input.turn.clamp(-1.0, 1.0),
        turret_yaw_delta: input.turret_yaw_delta,
        barrel_pitch_delta: input.barrel_pitch_delta,
        fire_pressed: input.fire_pressed,
        fire_just_pressed: input.fire_just_pressed,
        artillery_active: input.artillery_active,
    }
}

pub(super) fn read_single_local_intent(
    local_player_intent_q: &Query<&PlayerIntent, (With<Player>, With<LocalPlayer>)>,
) -> Option<PlayerIntent> {
    let mut intents = local_player_intent_q.iter();
    let first = intents.next().copied()?;
    if intents.next().is_some() {
        return None;
    }
    Some(first)
}

pub(super) fn intent_changed_significantly(prev: PlayerIntent, next: PlayerIntent) -> bool {
    const EPS: f32 = 0.001;
    (prev.throttle - next.throttle).abs() > EPS
        || (prev.turn - next.turn).abs() > EPS
        || (prev.turret_yaw_delta - next.turret_yaw_delta).abs() > EPS
        || (prev.barrel_pitch_delta - next.barrel_pitch_delta).abs() > EPS
        || prev.fire_pressed != next.fire_pressed
        || prev.fire_just_pressed != next.fire_just_pressed
        || prev.artillery_active != next.artillery_active
}
