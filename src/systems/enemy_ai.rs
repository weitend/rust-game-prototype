use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        enemy::{Enemy, EnemyAi, EnemyAiState, EnemyControllerState},
        fire_control::FireControl,
        intent::EnemyIntent,
        player::{LocalPlayer, Player},
        shoot_origin::ShootOrigin,
        shot_tracer::{ShotTracer, ShotTracerLifetime},
        weapon::HitscanWeapon,
    },
    resources::{local_player::LocalPlayerContext, tracer_assets::TracerAssets},
    systems::impact::ImpactEvent,
    utils::local_player::resolve_local_player_entity,
};

const ENEMY_SPEED: f32 = 10.2;
const ENEMY_ACCEL: f32 = 16.0;
const ENEMY_BRAKE: f32 = 22.0;
const ENEMY_YAW_SPEED: f32 = 4.4;
const ENEMY_YAW_ACCEL: f32 = 14.0;
const ENEMY_YAW_DAMPING: f32 = 10.0;
const ENEMY_GRAVITY: f32 = 13.0;
const ENEMY_MAX_FALL_SPEED: f32 = 35.0;
const ENEMY_EYE_HEIGHT: f32 = 0.45;
const PLAYER_AIM_HEIGHT: f32 = 0.4;

pub fn enemy_ai_state_system(
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    mut enemies: Query<(Entity, &Transform, &mut EnemyAi), With<Enemy>>,
    player_q: Query<&Transform, With<Player>>,
    rapier_context: ReadRapierContext,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        return;
    };
    let Ok(player_tf) = player_q.get(player_entity) else {
        return;
    };
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    for (enemy_entity, enemy_tf, mut ai) in &mut enemies {
        let to_player = player_tf.translation - enemy_tf.translation;
        let planar_distance = Vec2::new(to_player.x, to_player.z).length();

        if planar_distance > ai.detection_range {
            ai.state = EnemyAiState::Idle;
            continue;
        }

        let ray_origin = enemy_tf.translation + Vec3::Y * ENEMY_EYE_HEIGHT;
        let ray_target = player_tf.translation + Vec3::Y * PLAYER_AIM_HEIGHT;
        let ray_dir = (ray_target - ray_origin).normalize_or_zero();

        let has_los = if ray_dir == Vec3::ZERO {
            false
        } else {
            let filter = QueryFilter::new()
                .exclude_collider(enemy_entity)
                .exclude_rigid_body(enemy_entity)
                .exclude_sensors();

            rapier_context
                .cast_ray(ray_origin, ray_dir, ai.detection_range, true, filter)
                .is_some_and(|(hit_entity, _)| hit_entity == player_entity)
        };

        ai.state = if has_los && planar_distance <= ai.attack_range {
            EnemyAiState::Attack
        } else {
            EnemyAiState::Chase
        };
    }
}

pub fn enemy_move_system(
    time: Res<Time>,
    mut enemies: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &mut EnemyControllerState,
            Option<&KinematicCharacterControllerOutput>,
            &EnemyIntent,
        ),
        (With<Enemy>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();

    for (mut enemy_tf, mut controller, mut motor_state, output, intent) in &mut enemies {
        if let Some(yaw) = intent.look_yaw {
            let current_yaw = yaw_from_rotation(enemy_tf.rotation);
            let yaw_error = normalize_angle(yaw - current_yaw);
            let target_yaw_velocity = yaw_error.clamp(-1.0, 1.0) * ENEMY_YAW_SPEED;
            let yaw_delta = target_yaw_velocity - motor_state.yaw_velocity;
            let yaw_step = ENEMY_YAW_ACCEL * dt;
            motor_state.yaw_velocity += yaw_delta.clamp(-yaw_step, yaw_step);
        } else {
            let damping = (1.0 - ENEMY_YAW_DAMPING * dt).clamp(0.0, 1.0);
            motor_state.yaw_velocity *= damping;
        }

        enemy_tf.rotate_y(motor_state.yaw_velocity * dt);

        let target_planar_velocity = intent.move_dir.normalize_or_zero() * ENEMY_SPEED;
        let planar_delta = target_planar_velocity - motor_state.planar_velocity;
        let accel_rate = if target_planar_velocity == Vec3::ZERO {
            ENEMY_BRAKE
        } else if motor_state.planar_velocity.length_squared() > f32::EPSILON
            && motor_state
                .planar_velocity
                .normalize_or_zero()
                .dot(target_planar_velocity.normalize_or_zero())
                < 0.0
        {
            ENEMY_BRAKE
        } else {
            ENEMY_ACCEL
        };

        let max_step = accel_rate * dt;
        let delta_len = planar_delta.length();
        if delta_len <= max_step || delta_len <= f32::EPSILON {
            motor_state.planar_velocity = target_planar_velocity;
        } else {
            motor_state.planar_velocity += planar_delta / delta_len * max_step;
        }

        let mut horizontal = motor_state.planar_velocity * dt;
        horizontal.y = 0.0;

        let grounded = output.is_some_and(|o| o.grounded);
        if grounded {
            motor_state.vertical_velocity = 0.0;
        } else {
            motor_state.vertical_velocity =
                (motor_state.vertical_velocity - ENEMY_GRAVITY * dt).max(-ENEMY_MAX_FALL_SPEED);
        }

        controller.translation = Some(horizontal + Vec3::Y * motor_state.vertical_velocity * dt);
    }
}

fn normalize_angle(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

fn yaw_from_rotation(rotation: Quat) -> f32 {
    let (yaw, _, _) = rotation.to_euler(EulerRot::YXZ);
    normalize_angle(yaw)
}

pub fn enemy_fire_system(
    mut commands: Commands,
    mut impact_events: MessageWriter<ImpactEvent>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &ShootOrigin,
            &mut FireControl,
            &HitscanWeapon,
            &EnemyIntent,
        ),
        With<Enemy>,
    >,
    rapier_context: ReadRapierContext,
    tracer_assets: Res<TracerAssets>,
    time: Res<Time>,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    for (enemy_entity, enemy_tf, shoot_origin, mut fire_control, weapon, intent) in &mut enemies {
        if !intent.fire {
            fire_control.cooldown.reset();
            continue;
        }

        fire_control.cooldown.tick(time.delta());
        if !fire_control.cooldown.just_finished() {
            continue;
        }

        let ray_origin = enemy_tf.translation + enemy_tf.rotation * shoot_origin.muzzle_offset;
        let ray_target = intent.aim_target;
        let ray_dir = (ray_target - ray_origin).normalize_or_zero();
        if ray_dir == Vec3::ZERO {
            continue;
        }

        let filter = QueryFilter::new()
            .exclude_collider(enemy_entity)
            .exclude_rigid_body(enemy_entity)
            .exclude_sensors();

        let ray_result =
            rapier_context.cast_ray_and_get_normal(ray_origin, ray_dir, weapon.range, true, filter);

        let travel_distance = ray_result
            .map(|(_, hit)| hit.time_of_impact)
            .unwrap_or(weapon.range)
            .max(0.0);
        let tracer_speed = tracer_assets.speed.max(1.0);
        let tracer_lifetime = (travel_distance / tracer_speed).max(0.01);

        commands.spawn((
            Mesh3d(tracer_assets.mesh.clone()),
            MeshMaterial3d(tracer_assets.material.clone()),
            Transform::from_translation(ray_origin),
            ShotTracer {
                velocity: ray_dir * tracer_speed,
            },
            ShotTracerLifetime {
                timer: Timer::from_seconds(tracer_lifetime, TimerMode::Once),
            },
        ));

        let Some((hit_entity, hit)) = ray_result else {
            continue;
        };

        impact_events.write(ImpactEvent {
            source: Some(enemy_entity),
            target: hit_entity,
            point: hit.point,
            normal: hit.normal,
            damage: weapon.damage,
        });
    }
}
