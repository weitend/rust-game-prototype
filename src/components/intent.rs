use bevy::{math::Vec3, prelude::Component};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PlayerIntent {
    pub throttle: f32,
    pub turn: f32,
    pub turret_yaw_delta: f32,
    pub barrel_pitch_delta: f32,
    pub fire_pressed: bool,
    pub fire_just_pressed: bool,
    pub artillery_active: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct EnemyIntent {
    pub move_dir: Vec3,
    pub look_yaw: Option<f32>,
    pub fire: bool,
    pub aim_target: Vec3,
}

impl Default for EnemyIntent {
    fn default() -> Self {
        Self {
            move_dir: Vec3::ZERO,
            look_yaw: None,
            fire: false,
            aim_target: Vec3::ZERO,
        }
    }
}
