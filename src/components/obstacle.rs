use bevy::prelude::Component;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObstacleNetId(pub u64);
