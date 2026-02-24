use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayerMotionSettings {
    pub forward_speed: f32,
    pub reverse_speed: f32,
    pub yaw_speed: f32,
    pub drive_accel: f32,
    pub drive_brake: f32,
    pub yaw_accel: f32,
    pub yaw_damping: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub controller_offset: f32,
    pub autostep_height: f32,
    pub autostep_min_width: f32,
    pub snap_to_ground: f32,
}

impl Default for PlayerMotionSettings {
    fn default() -> Self {
        Self {
            forward_speed: 8.0,
            reverse_speed: 5.0,
            yaw_speed: 2.1,
            drive_accel: 18.0,
            drive_brake: 24.0,
            yaw_accel: 8.5,
            yaw_damping: 9.0,
            gravity: 13.0,
            max_fall_speed: 35.0,
            controller_offset: 0.003,
            autostep_height: 0.34,
            autostep_min_width: 0.2,
            snap_to_ground: 0.05,
        }
    }
}
