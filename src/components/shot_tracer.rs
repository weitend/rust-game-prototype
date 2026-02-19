use bevy::{math::Vec3, prelude::Component, time::Timer};

#[derive(Component)]
pub struct ShotTracer {
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct ShotTracerLifetime {
    pub timer: Timer,
}
