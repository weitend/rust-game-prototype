use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::{components::player::Player, resources::input_settings::InputSettings};

pub fn player_rotate_system(
    mut query: Query<&mut Transform, With<Player>>,
    mut motion: MessageReader<MouseMotion>,
    input_settings: Res<InputSettings>,
) {
    let mut player = query.single_mut().expect("Failed to find Player Transform");

    let mut delta_x: f32 = 0.0;
    for ev in motion.read() {
        delta_x += ev.delta.x;
    }

    if delta_x != 0.0 {
        player.rotate_y(-delta_x * input_settings.mouse_sensitivity);
    }
}
