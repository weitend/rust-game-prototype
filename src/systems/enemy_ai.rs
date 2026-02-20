use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::Health,
        enemy::{Enemy, EnemyAi, EnemyAiState, EnemyControllerState},
        fire_control::FireControl,
        impact_mark_lifetime::ImpactMarkLifetime,
        obstacle::Obstacle,
        player::Player,
        shoot_origin::ShootOrigin,
        shot_tracer::{ShotTracer, ShotTracerLifetime},
        weapon::HitscanWeapon,
    },
    resources::{impact_assets::ImpactAssets, tracer_assets::TracerAssets},
    systems::combat::DamageEvent,
};

const ENEMY_SPEED: f32 = 10.2;
const ENEMY_GRAVITY: f32 = 13.0;
const ENEMY_MAX_FALL_SPEED: f32 = 35.0;
const ENEMY_EYE_HEIGHT: f32 = 0.45;
const PLAYER_AIM_HEIGHT: f32 = 0.4;

pub fn enemy_ai_state_system(
    mut enemies: Query<(Entity, &Transform, &mut EnemyAi), With<Enemy>>,
    player_q: Query<(Entity, &Transform), With<Player>>,
    rapier_context: ReadRapierContext,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
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
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &mut EnemyControllerState,
            Option<&KinematicCharacterControllerOutput>,
            &EnemyAi,
        ),
        (With<Enemy>, Without<Player>),
    >,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    let dt = time.delta_secs();

    for (mut enemy_tf, mut controller, mut motor_state, output, ai) in &mut enemies {
        let mut to_player = player_tf.translation - enemy_tf.translation;
        to_player.y = 0.0;
        let look_dir = to_player.normalize_or_zero();

        if look_dir != Vec3::ZERO {
            let yaw = look_dir.x.atan2(-look_dir.z);
            enemy_tf.rotation = Quat::from_rotation_y(yaw);
        }

        let mut horizontal = Vec3::ZERO;
        if ai.state == EnemyAiState::Chase {
            horizontal = look_dir * ENEMY_SPEED * dt;
        }
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

pub fn enemy_fire_system(
    mut commands: Commands,
    mut damage_events: MessageWriter<DamageEvent>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &ShootOrigin,
            &mut FireControl,
            &HitscanWeapon,
            &EnemyAi,
        ),
        With<Enemy>,
    >,
    player_q: Query<&Transform, With<Player>>,
    obstacles: Query<(), With<Obstacle>>,
    damageable_targets: Query<(), With<Health>>,
    rapier_context: ReadRapierContext,
    impact_assets: Res<ImpactAssets>,
    tracer_assets: Res<TracerAssets>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    for (enemy_entity, enemy_tf, shoot_origin, mut fire_control, weapon, ai) in &mut enemies {
        if ai.state != EnemyAiState::Attack {
            fire_control.cooldown.reset();
            continue;
        }

        fire_control.cooldown.tick(time.delta());
        if !fire_control.cooldown.just_finished() {
            continue;
        }

        let ray_origin = enemy_tf.translation + enemy_tf.rotation * shoot_origin.muzzle_offset;
        let ray_target = player_tf.translation + Vec3::Y * PLAYER_AIM_HEIGHT;
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

        if damageable_targets.contains(hit_entity) {
            damage_events.write(DamageEvent {
                source: Some(enemy_entity),
                target: hit_entity,
                amount: weapon.damage,
            });
        }

        if obstacles.contains(hit_entity) {
            commands.spawn((
                Mesh3d(impact_assets.mesh.clone()),
                MeshMaterial3d(impact_assets.material.clone()),
                Transform::from_translation(hit.point + hit.normal * (impact_assets.radius * 0.35)),
                ImpactMarkLifetime {
                    timer: Timer::from_seconds(impact_assets.lifetime_secs, TimerMode::Once),
                },
            ));
        }
    }
}
