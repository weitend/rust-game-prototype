use bevy::{prelude::Component, time::Timer};

#[derive(Component)]
pub struct DebrisChip {
    pub timer: Timer,
}
