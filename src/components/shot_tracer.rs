use bevy::{math::Vec3, prelude::Component, time::Timer};

#[derive(Component)]
pub struct ShotTracer {
    pub velocity: Vec3,
    pub smoke_timer: Timer,
}

#[derive(Component)]
pub struct ShotTracerLifetime {
    pub timer: Timer,
}

#[derive(Component)]
pub struct SmokePuff {
    pub velocity: Vec3,
    pub timer: Timer,
    pub start_scale: f32,
    pub end_scale: f32,
}

#[derive(Component)]
pub struct ExplosionVfx {
    pub timer: Timer,
    pub start_scale: f32,
    pub end_scale: f32,
}
