use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Debug)]
pub struct EnemyMotionSettings {
    pub speed: f32,
    pub accel: f32,
    pub brake: f32,
    pub yaw_speed: f32,
    pub yaw_accel: f32,
    pub yaw_damping: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub eye_height: f32,
    pub target_height: f32,
}

impl Default for EnemyMotionSettings {
    fn default() -> Self {
        Self {
            speed: 10.2,
            accel: 16.0,
            brake: 22.0,
            yaw_speed: 4.4,
            yaw_accel: 14.0,
            yaw_damping: 10.0,
            gravity: 13.0,
            max_fall_speed: 35.0,
            eye_height: 0.45,
            target_height: 0.4,
        }
    }
}
