use bevy::{ecs::entity::Entity, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectileKind {
    Shell,
    Rocket,
    Plasma,
    Custom(u16),
}

#[derive(Clone, Copy, Debug)]
pub enum ProjectileImpactMode {
    Direct,
    Explosion { radius: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileParams {
    pub kind: ProjectileKind,
    pub damage: f32,
    pub gravity: f32,
    pub max_lifetime_secs: f32,
    pub max_distance: f32,
    pub collision_radius: f32,
    pub impact_mode: ProjectileImpactMode,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectileState {
    pub velocity: Vec3,
    pub traveled_distance: f32,
    pub lived_secs: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Projectile {
    pub source: Option<Entity>,
    pub params: ProjectileParams,
    pub state: ProjectileState,
}

impl Projectile {
    pub fn with_params(source: Option<Entity>, params: ProjectileParams, velocity: Vec3) -> Self {
        Self {
            source,
            params,
            state: ProjectileState {
                velocity,
                traveled_distance: 0.0,
                lived_secs: 0.0,
            },
        }
    }

    pub fn new(
        source: Option<Entity>,
        damage: f32,
        velocity: Vec3,
        gravity: f32,
        lifetime_secs: f32,
        max_distance: f32,
    ) -> Self {
        Self::with_params(
            source,
            ProjectileParams {
                kind: ProjectileKind::Shell,
                damage,
                gravity,
                max_lifetime_secs: lifetime_secs,
                max_distance,
                collision_radius: 0.0,
                impact_mode: ProjectileImpactMode::Direct,
            },
            velocity,
        )
    }
}
