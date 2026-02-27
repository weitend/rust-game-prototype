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
    pub engine_rpm: f32,
    pub transmission_gear: i8,
    pub left_track_angular_speed: f32,
    pub right_track_angular_speed: f32,
    pub ground_speed_forward: f32,
    pub left_track_slip_ratio: f32,
    pub right_track_slip_ratio: f32,
    pub mean_contact_fx: f32,
    pub mean_contact_fy: f32,
}
