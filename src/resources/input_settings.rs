use bevy::prelude::Resource;

#[derive(Resource)]
pub struct InputSettings {
    pub mouse_sensitivity: f32,
}
