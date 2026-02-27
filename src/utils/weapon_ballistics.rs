use crate::{
    components::{
        projectile::ProjectileParams,
        weapon::{HitscanWeapon, ProjectileWeaponProfile},
    },
    resources::aim_settings::AimSettings,
};

pub const DEFAULT_DIRECT_PROJECTILE_SPEED: f32 = 650.0;

#[derive(Clone, Copy, Debug)]
pub struct ProjectileSpawnParams {
    pub params: ProjectileParams,
    pub initial_speed: f32,
}

pub fn build_projectile_spawn_params(
    weapon: &HitscanWeapon,
    profile: ProjectileWeaponProfile,
    artillery_active: bool,
    aim_settings: &AimSettings,
) -> ProjectileSpawnParams {
    if artillery_active {
        let defaults = aim_settings.artillery_ballistic_params(weapon.range);
        let speed = profile
            .artillery
            .speed
            .unwrap_or(defaults.initial_speed)
            .max(1.0);
        let gravity = profile
            .artillery
            .gravity
            .unwrap_or(defaults.gravity)
            .max(0.0);
        let max_distance = profile
            .artillery
            .max_distance
            .unwrap_or(defaults.max_distance)
            .max(0.1);
        let lifetime = profile
            .artillery
            .max_lifetime_secs
            .unwrap_or((max_distance / speed).clamp(0.05, 12.0))
            .max(0.01);
        let collision_radius = profile
            .artillery
            .collision_radius
            .unwrap_or(defaults.collision_radius)
            .max(0.0);
        return ProjectileSpawnParams {
            params: ProjectileParams {
                kind: profile.kind,
                damage: weapon.damage,
                gravity,
                max_lifetime_secs: lifetime,
                max_distance,
                collision_radius,
                impact_mode: profile.artillery.impact_mode,
            },
            initial_speed: speed,
        };
    }

    let speed = profile
        .direct
        .speed
        .unwrap_or(DEFAULT_DIRECT_PROJECTILE_SPEED)
        .max(1.0);
    let gravity = profile.direct.gravity.unwrap_or(0.0).max(0.0);
    let max_distance = profile.direct.max_distance.unwrap_or(weapon.range).max(0.1);
    let lifetime = profile
        .direct
        .max_lifetime_secs
        .unwrap_or((max_distance / speed).clamp(0.02, 8.0))
        .max(0.01);
    let collision_radius = profile.direct.collision_radius.unwrap_or(0.0).max(0.0);
    ProjectileSpawnParams {
        params: ProjectileParams {
            kind: profile.kind,
            damage: weapon.damage,
            gravity,
            max_lifetime_secs: lifetime,
            max_distance,
            collision_radius,
            impact_mode: profile.direct.impact_mode,
        },
        initial_speed: speed,
    }
}
