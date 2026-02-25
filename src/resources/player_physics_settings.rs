use bevy::prelude::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerHullPhysicsMode {
    KinematicController,
    DynamicForces,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayerPhysicsSettings {
    pub mode: PlayerHullPhysicsMode,
    pub dynamic_linear_damping: f32,
    pub dynamic_angular_damping: f32,
    pub dynamic_drive_accel_gain: f32,
    pub dynamic_drive_accel_max: f32,
    pub dynamic_lateral_grip: f32,
    pub dynamic_yaw_accel_gain: f32,
    pub dynamic_yaw_accel_max: f32,
}

impl Default for PlayerPhysicsSettings {
    fn default() -> Self {
        let mode = match std::env::var("RUST_GAME_PLAYER_HULL_MODE").ok().as_deref() {
            Some("dynamic") => PlayerHullPhysicsMode::DynamicForces,
            _ => PlayerHullPhysicsMode::KinematicController,
        };

        Self {
            mode,
            dynamic_linear_damping: 1.2,
            dynamic_angular_damping: 2.4,
            dynamic_drive_accel_gain: 7.2,
            dynamic_drive_accel_max: 20.0,
            dynamic_lateral_grip: 6.0,
            dynamic_yaw_accel_gain: 8.0,
            dynamic_yaw_accel_max: 18.0,
        }
    }
}
