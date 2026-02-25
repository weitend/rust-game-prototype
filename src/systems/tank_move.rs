use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    ExternalForce, KinematicCharacterController, KinematicCharacterControllerOutput,
    ReadMassProperties, Velocity,
};

use crate::{
    components::{
        intent::PlayerIntent,
        player::{LocalPlayer, Player, PlayerControllerState},
        tank::TankHull,
    },
    resources::{
        local_player::LocalPlayerContext,
        player_motion_settings::PlayerMotionSettings,
        player_physics_settings::{PlayerHullPhysicsMode, PlayerPhysicsSettings},
    },
    utils::local_player::resolve_local_player_entity,
};

pub fn tank_hull_move_system(
    time: Res<Time>,
    settings: Res<PlayerMotionSettings>,
    physics_settings: Res<PlayerPhysicsSettings>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut player_q: Query<
        (
            &mut Transform,
            Option<&mut KinematicCharacterController>,
            Option<&Velocity>,
            Option<&mut ExternalForce>,
            Option<&ReadMassProperties>,
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
    let Ok((mut player_tf, controller, velocity, external_force, mass, mut state, intent, output)) =
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

    match physics_settings.mode {
        PlayerHullPhysicsMode::KinematicController => {
            let Some(mut controller) = controller else {
                warn!("Expected KinematicCharacterController for player in kinematic mode");
                return;
            };
            if let Some(mut external_force) = external_force {
                external_force.force = Vec3::ZERO;
                external_force.torque = Vec3::ZERO;
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
        PlayerHullPhysicsMode::DynamicForces => {
            let Some(velocity) = velocity else {
                warn!("Expected Velocity for player in dynamic mode");
                return;
            };
            let Some(mut external_force) = external_force else {
                warn!("Expected ExternalForce for player in dynamic mode");
                return;
            };
            let mass = mass.map(|m| m.mass.max(0.1)).unwrap_or(1.0);
            if let Some(mut controller) = controller {
                controller.translation = None;
            }

            let forward = (player_tf.rotation * -Vec3::Z)
                .with_y(0.0)
                .normalize_or_zero();
            let planar_velocity = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
            let current_forward_speed = planar_velocity.dot(forward);
            let lateral_velocity = planar_velocity - forward * current_forward_speed;

            let forward_accel = ((state.drive_velocity - current_forward_speed)
                * physics_settings.dynamic_drive_accel_gain)
                .clamp(
                    -physics_settings.dynamic_drive_accel_max,
                    physics_settings.dynamic_drive_accel_max,
                );
            let forward_force = forward * (forward_accel * mass);
            let lateral_force = -lateral_velocity * (physics_settings.dynamic_lateral_grip * mass);
            external_force.force = forward_force + lateral_force;
            external_force.force.y = 0.0;

            let yaw_accel = ((state.yaw_velocity - velocity.angvel.y)
                * physics_settings.dynamic_yaw_accel_gain)
                .clamp(
                    -physics_settings.dynamic_yaw_accel_max,
                    physics_settings.dynamic_yaw_accel_max,
                );
            external_force.torque = Vec3::Y * (yaw_accel * mass);
            state.vertical_velocity = velocity.linvel.y;
        }
    }
}
