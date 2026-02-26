use bevy::prelude::*;

use crate::utils::ballistics::BallisticParams;

pub const AIM_MARKER_RENDER_LAYER: usize = 1;

#[derive(Resource, Clone, Copy, Debug)]
pub struct AimSettings {
    pub marker_radius: f32,
    pub marker_height: f32,
    pub marker_surface_offset: f32,
    pub range_fallback: f32,
    pub artillery_pitch_min: f32,
    pub artillery_pitch_max: f32,
    pub artillery_effective_pitch_max: f32,
    pub artillery_auto_raise_speed: f32,
    pub artillery_camera_height: f32,
    pub artillery_camera_back: f32,
    pub artillery_camera_back_pitch_extra: f32,
    pub artillery_camera_height_pitch_extra: f32,
    pub artillery_camera_look_up: f32,
    pub artillery_camera_look_forward: f32,
    pub artillery_camera_smooth: f32,
    pub artillery_projectile_speed: f32,
    pub artillery_gravity: f32,
    pub artillery_step_secs: f32,
    pub artillery_max_steps: usize,
    pub artillery_max_distance: f32,
    pub artillery_downcast_distance: f32,
    pub artillery_min_safe_distance: f32,
    pub vignette_border_px: f32,
    pub vignette_alpha: f32,
}

impl Default for AimSettings {
    fn default() -> Self {
        Self {
            marker_radius: 0.24,
            marker_height: 0.02,
            marker_surface_offset: 0.02,
            range_fallback: 45.0,
            artillery_pitch_min: 20.0_f32.to_radians(),
            artillery_pitch_max: 62.0_f32.to_radians(),
            artillery_effective_pitch_max: 45.0_f32.to_radians(),
            artillery_auto_raise_speed: 1.65,
            artillery_camera_height: 24.0,
            artillery_camera_back: 7.5,
            artillery_camera_back_pitch_extra: 13.0,
            artillery_camera_height_pitch_extra: 4.5,
            artillery_camera_look_up: 1.8,
            artillery_camera_look_forward: 7.0,
            artillery_camera_smooth: 3.6,
            artillery_projectile_speed: 42.0,
            artillery_gravity: 18.0,
            artillery_step_secs: 0.05,
            artillery_max_steps: 240,
            artillery_max_distance: 180.0,
            artillery_downcast_distance: 180.0,
            artillery_min_safe_distance: 0.6,
            vignette_border_px: 140.0,
            vignette_alpha: 0.20,
        }
    }
}

impl AimSettings {
    pub fn artillery_pitch_limit(self) -> f32 {
        self.artillery_pitch_max
            .min(self.artillery_effective_pitch_max)
    }

    pub fn effective_range(self, weapon_range: f32) -> f32 {
        self.artillery_max_distance
            .max(weapon_range.max(self.range_fallback))
    }

    pub fn artillery_ballistic_params(self, weapon_range: f32) -> BallisticParams {
        BallisticParams {
            initial_speed: self.artillery_projectile_speed,
            gravity: self.artillery_gravity,
            step_secs: self.artillery_step_secs,
            max_steps: self.artillery_max_steps,
            max_distance: self.effective_range(weapon_range),
            collision_radius: 0.0,
            downcast_distance: self.artillery_downcast_distance,
            min_safe_distance: self.artillery_min_safe_distance,
        }
    }
}
