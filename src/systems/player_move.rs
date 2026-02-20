use bevy::prelude::*;
use bevy_rapier3d::prelude::{KinematicCharacterController, KinematicCharacterControllerOutput};

use crate::components::player::{Player, PlayerControllerState};

pub fn player_move_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &Transform,
            &mut KinematicCharacterController,
            &mut PlayerControllerState,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    const SPEED: f32 = 8.0;
    const GRAVITY: f32 = 13.0;
    const JUMP_SPEED: f32 = 9.2;
    const MAX_FALL_SPEED: f32 = 35.0;

    let mut direction = Vec3::ZERO;
    let Ok((player_tf, mut controller, mut state, output)) = query.single_mut() else {
        return;
    };

    let mut forward = player_tf.rotation * -Vec3::Z;
    let mut right = player_tf.rotation * Vec3::X;
    forward.y = 0.0;
    right.y = 0.0;
    forward = forward.normalize_or_zero();
    right = right.normalize_or_zero();

    if keyboard.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += right;
    }

    let dt = time.delta_secs();
    let mut horizontal = direction.normalize_or_zero() * SPEED * dt;
    horizontal.y = 0.0;

    let grounded = output.is_some_and(|o| o.grounded);
    if grounded {
        if keyboard.just_pressed(KeyCode::Space) {
            state.vertical_velocity = JUMP_SPEED;
        } else {
            state.vertical_velocity = 0.0;
        }
    } else {
        state.vertical_velocity = (state.vertical_velocity - GRAVITY * dt).max(-MAX_FALL_SPEED);
    }

    controller.translation = Some(horizontal + Vec3::Y * state.vertical_velocity * dt);
}
