use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::components::{
    enemy::{Enemy, EnemyAi, EnemyAiState},
    intent::{EnemyIntent, PlayerIntent},
    player::{LocalPlayer, Player},
};
use crate::resources::{
    enemy_motion_settings::EnemyMotionSettings, local_player::LocalPlayerContext,
};
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

    intent.throttle = axis_pressed(&keyboard, KeyCode::KeyW, KeyCode::KeyS).clamp(-1.0, 1.0);
    intent.turn = axis_pressed(&keyboard, KeyCode::KeyA, KeyCode::KeyD).clamp(-1.0, 1.0);
    intent.turret_yaw_delta = look_delta.x;
    intent.barrel_pitch_delta = look_delta.y;
    intent.fire_pressed = mouse_buttons.pressed(MouseButton::Left);
    intent.fire_just_pressed = mouse_buttons.just_pressed(MouseButton::Left);
    intent.artillery_active = mouse_buttons.pressed(MouseButton::Right);
}

pub fn enemy_intent_from_ai_system(
    enemy_motion: Res<EnemyMotionSettings>,
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    player_q: Query<&Transform, With<Player>>,
    mut enemies: Query<(&Transform, &EnemyAi, &mut EnemyIntent), With<Enemy>>,
) {
    let player_aim_target = resolve_local_player_entity(&local_player_ctx, &local_player_q)
        .and_then(|player_entity| player_q.get(player_entity).ok())
        .map(|player_tf| player_tf.translation + Vec3::Y * enemy_motion.target_height);

    for (enemy_tf, ai, mut intent) in &mut enemies {
        let mut next = EnemyIntent::default();

        let Some(aim_target) = player_aim_target else {
            *intent = next;
            continue;
        };

        let mut to_target = aim_target - enemy_tf.translation;
        to_target.y = 0.0;
        let look_dir = to_target.normalize_or_zero();
        if look_dir != Vec3::ZERO {
            next.look_yaw = Some(look_dir.x.atan2(-look_dir.z));
        }

        if ai.state == EnemyAiState::Chase {
            next.move_dir = look_dir;
        }
        next.fire = ai.state == EnemyAiState::Attack;
        next.aim_target = aim_target;

        *intent = next;
    }
}

fn axis_pressed(input: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    let pos = input.pressed(positive) as u8 as f32;
    let neg = input.pressed(negative) as u8 as f32;
    pos - neg
}
