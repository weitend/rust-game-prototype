use bevy::prelude::Component;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Default)]
pub struct PlayerControllerState {
    pub vertical_velocity: f32,
    pub drive_velocity: f32,
    pub yaw_velocity: f32,
}
