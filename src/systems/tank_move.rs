use bevy::prelude::*;
use bevy_rapier3d::prelude::{KinematicCharacterController, KinematicCharacterControllerOutput};

use crate::{
    components::{
        player::{Player, PlayerControllerState},
        tank::TankHull,
    },
    resources::tank_settings::TankSettings,
};

pub fn tank_hull_move_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<TankSettings>,
    mut player_q: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &mut PlayerControllerState,
            Option<&KinematicCharacterControllerOutput>,
        ),
        (With<Player>, With<TankHull>),
    >,
) {
    let Ok((mut player_tf, mut controller, mut state, output)) = player_q.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let throttle_axis = axis_pressed(&keyboard, KeyCode::KeyW, KeyCode::KeyS).clamp(-1.0, 1.0);
    let turn_axis = axis_pressed(&keyboard, KeyCode::KeyA, KeyCode::KeyD).clamp(-1.0, 1.0);

    player_tf.rotate_y(turn_axis * settings.yaw_speed * dt);

    let forward = (player_tf.rotation * -Vec3::Z)
        .with_y(0.0)
        .normalize_or_zero();

    let drive_speed = if throttle_axis >= 0.0 {
        settings.forward_speed
    } else {
        settings.reverse_speed
    };

    let grounded = output.is_some_and(|o| o.grounded);
    if grounded {
        state.vertical_velocity = 0.0;
    } else {
        state.vertical_velocity =
            (state.vertical_velocity - settings.gravity * dt).max(-settings.max_fall_speed);
    }

    let horizontal = forward * (throttle_axis * drive_speed * dt);
    controller.translation = Some(horizontal + Vec3::Y * state.vertical_velocity * dt);
}

fn axis_pressed(input: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    let pos = input.pressed(positive) as u8 as f32;
    let neg = input.pressed(negative) as u8 as f32;
    pos - neg
}
