use bevy::prelude::Component;

#[derive(Component, Clone, Copy, Debug)]
pub struct HitscanWeapon {
    pub damage: f32,
    pub range: f32,
}
