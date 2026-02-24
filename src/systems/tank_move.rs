use bevy::prelude::*;
use bevy_rapier3d::prelude::{KinematicCharacterController, KinematicCharacterControllerOutput};

use crate::{
    components::{
        intent::PlayerIntent,
        player::{LocalPlayer, Player, PlayerControllerState},
        tank::TankHull,
    },
    resources::{local_player::LocalPlayerContext, tank_settings::TankSettings},
    utils::local_player::resolve_local_player_entity,
};

pub fn tank_hull_move_system(
    time: Res<Time>,
    settings: Res<TankSettings>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut player_q: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &mut PlayerControllerState,
            &PlayerIntent,
            Option<&KinematicCharacterControllerOutput>,
        ),
        (With<Player>, With<TankHull>),
    >,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok((mut player_tf, mut controller, mut state, intent, output)) =
        player_q.get_mut(player_entity)
    else {
        return;
    };

    let dt = time.delta_secs();
    let throttle_axis = intent.throttle;
    let turn_axis = intent.turn;

    let target_drive_velocity = if throttle_axis >= 0.0 {
        throttle_axis * settings.forward_speed
    } else {
        throttle_axis * settings.reverse_speed
    };
    let drive_delta = target_drive_velocity - state.drive_velocity;
    let drive_rate = if throttle_axis.abs() <= f32::EPSILON {
        settings.drive_brake * 0.75
    } else if state.drive_velocity.signum() != target_drive_velocity.signum()
        && state.drive_velocity.abs() > f32::EPSILON
    {
        settings.drive_brake
    } else {
        settings.drive_accel
    };
    let drive_step = drive_rate * dt;
    state.drive_velocity += drive_delta.clamp(-drive_step, drive_step);

    let target_yaw_velocity = turn_axis * settings.yaw_speed;
    let yaw_delta = target_yaw_velocity - state.yaw_velocity;
    let yaw_step = settings.yaw_accel * dt;
    state.yaw_velocity += yaw_delta.clamp(-yaw_step, yaw_step);
    if turn_axis.abs() <= f32::EPSILON {
        let damping = (1.0 - settings.yaw_damping * dt).clamp(0.0, 1.0);
        state.yaw_velocity *= damping;
    }

    player_tf.rotate_y(state.yaw_velocity * dt);

    let forward = (player_tf.rotation * -Vec3::Z)
        .with_y(0.0)
        .normalize_or_zero();

    let grounded = output.is_some_and(|o| o.grounded);
    if grounded {
        state.vertical_velocity = 0.0;
    } else {
        state.vertical_velocity =
            (state.vertical_velocity - settings.gravity * dt).max(-settings.max_fall_speed);
    }

    let horizontal = forward * (state.drive_velocity * dt);
    controller.translation = Some(horizontal + Vec3::Y * state.vertical_velocity * dt);
}
