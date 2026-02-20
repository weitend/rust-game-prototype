use bevy::prelude::Component;

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyAiState {
    Idle,
    Chase,
    Attack,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct EnemyAi {
    pub state: EnemyAiState,
    pub detection_range: f32,
    pub attack_range: f32,
}

impl EnemyAi {
    pub fn new(detection_range: f32, attack_range: f32) -> Self {
        Self {
            state: EnemyAiState::Idle,
            detection_range,
            attack_range,
        }
    }
}

#[derive(Component, Default)]
pub struct EnemyControllerState {
    pub vertical_velocity: f32,
}
