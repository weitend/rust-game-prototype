use bevy::prelude::*;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct BulletLifeTime {
    pub timer: Timer,
}
