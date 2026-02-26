use bevy::prelude::Component;

use crate::components::projectile::{ProjectileImpactMode, ProjectileKind};

#[derive(Component, Clone, Copy, Debug)]
pub struct HitscanWeapon {
    pub damage: f32,
    pub range: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileBallisticsProfile {
    pub speed: Option<f32>,
    pub gravity: Option<f32>,
    pub max_distance: Option<f32>,
    pub max_lifetime_secs: Option<f32>,
    pub collision_radius: Option<f32>,
    pub impact_mode: ProjectileImpactMode,
}

impl Default for ProjectileBallisticsProfile {
    fn default() -> Self {
        Self {
            speed: None,
            gravity: None,
            max_distance: None,
            max_lifetime_secs: None,
            collision_radius: None,
            impact_mode: ProjectileImpactMode::Direct,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ProjectileWeaponProfile {
    pub kind: ProjectileKind,
    pub direct: ProjectileBallisticsProfile,
    pub artillery: ProjectileBallisticsProfile,
}

impl Default for ProjectileWeaponProfile {
    fn default() -> Self {
        Self {
            kind: ProjectileKind::Shell,
            direct: ProjectileBallisticsProfile {
                speed: Some(65.0),
                gravity: Some(0.0),
                ..Default::default()
            },
            artillery: ProjectileBallisticsProfile {
                speed: None,
                gravity: None,
                ..Default::default()
            },
        }
    }
}
