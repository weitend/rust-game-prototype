use bevy::{math::Vec3, prelude::Component};

#[derive(Component)]
pub struct ShootOrigin {
    pub muzzle_offset: Vec3,
}
