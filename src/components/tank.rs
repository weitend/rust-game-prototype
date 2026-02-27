use bevy::prelude::Vec3;
use bevy::prelude::{Component, Entity};

#[derive(Component)]
pub struct TankHull;

#[derive(Component)]
pub struct TankTurret;

#[derive(Component, Default)]
pub struct TankTurretState {
    pub initialized: bool,
    pub yaw: f32,
    pub yaw_target: f32,
    pub yaw_velocity: f32,
}

#[derive(Component)]
pub struct TankBarrel;

#[derive(Component, Default)]
pub struct TankBarrelState {
    pub initialized: bool,
    pub pitch: f32,
    pub pitch_target: f32,
    pub pitch_velocity: f32,
    pub artillery_mode_active: bool,
}

#[derive(Component)]
pub struct TankMuzzle;

#[derive(Component, Clone, Copy, Debug)]
pub struct TankParts {
    pub turret: Entity,
    pub barrel: Entity,
    pub muzzle: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct SuspensionPoint {
    pub local_anchor: Vec3,
    pub side: TrackSide,
}

#[derive(Component, Clone, Debug)]
pub struct TankSuspension {
    pub points: Vec<SuspensionPoint>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct GroundContact {
    pub grounded: bool,
    pub normal: Vec3,
    pub slope_angle: f32,
}

impl Default for GroundContact {
    fn default() -> Self {
        Self {
            grounded: false,
            normal: Vec3::Y,
            slope_angle: 0.0,
        }
    }
}
