use bevy::{prelude::Component, time::Timer};

#[derive(Component)]
pub struct FireControl {
    pub cooldown: Timer,
    pub shots_per_second: f32,
}
