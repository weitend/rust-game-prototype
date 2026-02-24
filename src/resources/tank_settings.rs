use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Debug)]
pub struct TankSettings {
    pub forward_speed: f32,
    pub reverse_speed: f32,
    pub yaw_speed: f32,
    pub turret_yaw_sensitivity: f32,
    pub turret_yaw_limit: f32,
    pub barrel_pitch_sensitivity: f32,
    pub barrel_pitch_min: f32,
    pub barrel_pitch_max: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub controller_offset: f32,
    pub autostep_height: f32,
    pub autostep_min_width: f32,
    pub snap_to_ground: f32,
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
            forward_speed: 8.0,
            reverse_speed: 5.0,
            yaw_speed: 2.1,
            turret_yaw_sensitivity: 0.0024,
            turret_yaw_limit: 170.0_f32.to_radians(),
            barrel_pitch_sensitivity: 0.0020,
            barrel_pitch_min: -5.0_f32.to_radians(),
            barrel_pitch_max: 25.0_f32.to_radians(),
            gravity: 13.0,
            max_fall_speed: 35.0,
            controller_offset: 0.003,
            autostep_height: 0.34,
            autostep_min_width: 0.2,
            snap_to_ground: 0.05,
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
