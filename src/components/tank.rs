use bevy::prelude::Component;

#[derive(Component)]
pub struct TankHull;

#[derive(Component)]
pub struct TankTurret;

#[derive(Component, Default)]
pub struct TankTurretState {
    pub yaw: f32,
}

#[derive(Component)]
pub struct TankBarrel;

#[derive(Component, Default)]
pub struct TankBarrelState {
    pub pitch: f32,
}

#[derive(Component)]
pub struct TankMuzzle;
