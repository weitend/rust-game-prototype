use bevy::{prelude::Component, time::Timer};

#[derive(Component)]
pub struct FireControl {
    pub cooldown: Timer,
}
