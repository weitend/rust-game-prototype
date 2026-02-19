use bevy::prelude::Component;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerControllerState {
    pub vertical_velocity: f32,
}
