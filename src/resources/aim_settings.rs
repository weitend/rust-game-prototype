use bevy::prelude::*;

pub const AIM_MARKER_RENDER_LAYER: usize = 1;

#[derive(Resource, Clone, Copy, Debug)]
pub struct AimSettings {
    pub marker_radius: f32,
    pub marker_height: f32,
    pub marker_surface_offset: f32,
    pub range_fallback: f32,
}

impl Default for AimSettings {
    fn default() -> Self {
        Self {
            marker_radius: 0.24,
            marker_height: 0.02,
            marker_surface_offset: 0.02,
            range_fallback: 45.0,
        }
    }
}
