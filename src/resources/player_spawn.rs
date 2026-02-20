use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct PlayerTemplate {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub muzzle_offset: Vec3,
    pub spawn_translation: Vec3,
    pub max_health: f32,
    pub shots_per_second: f32,
    pub weapon_damage: f32,
    pub weapon_range: f32,
}

#[derive(Resource)]
pub struct PlayerRespawnState {
    pub pending: bool,
    pub timer: Timer,
    pub delay_secs: f32,
}

impl Default for PlayerRespawnState {
    fn default() -> Self {
        Self {
            pending: false,
            timer: Timer::from_seconds(2.5, TimerMode::Once),
            delay_secs: 2.5,
        }
    }
}
