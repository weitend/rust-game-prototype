use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Debug)]
pub struct TankSettings {
    pub turret_yaw_sensitivity: f32,
    pub turret_yaw_limit: f32,
    pub turret_yaw_max_speed: f32,
    pub turret_yaw_accel: f32,
    pub turret_yaw_damping: f32,
    pub turret_yaw_tracking_gain: f32,
    pub barrel_pitch_sensitivity: f32,
    pub barrel_pitch_min: f32,
    pub barrel_pitch_max: f32,
    pub barrel_pitch_max_speed: f32,
    pub barrel_pitch_accel: f32,
    pub barrel_pitch_damping: f32,
    pub barrel_pitch_tracking_gain: f32,
    pub camera_offset_right: f32,
    pub camera_offset_up: f32,
    pub camera_offset_back: f32,
    pub camera_look_right: f32,
    pub camera_look_up: f32,
    pub camera_look_forward: f32,
    pub camera_follow_y: f32,
    pub camera_smooth: f32,
}

impl Default for TankSettings {
    fn default() -> Self {
        Self {
            turret_yaw_sensitivity: 0.0024,
            turret_yaw_limit: 170.0_f32.to_radians(),
            turret_yaw_max_speed: 4.8,
            turret_yaw_accel: 26.0,
            turret_yaw_damping: 11.0,
            turret_yaw_tracking_gain: 14.0,
            barrel_pitch_sensitivity: 0.0020,
            barrel_pitch_min: -5.0_f32.to_radians(),
            barrel_pitch_max: 25.0_f32.to_radians(),
            barrel_pitch_max_speed: 2.8,
            barrel_pitch_accel: 18.0,
            barrel_pitch_damping: 10.0,
            barrel_pitch_tracking_gain: 11.0,
            camera_offset_right: -1.0,
            camera_offset_up: 1.8,
            camera_offset_back: 6.0,
            camera_look_right: 0.2,
            camera_look_up: 0.2,
            camera_look_forward: 1.6,
            camera_follow_y: 0.8,
            camera_smooth: 5.0,
        }
    }
}
