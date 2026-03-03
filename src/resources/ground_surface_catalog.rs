use bevy::prelude::Resource;

use crate::components::ground_surface::GroundSurfaceKind;

#[derive(Clone, Copy, Debug)]
pub struct GroundSurfaceParams {
    pub friction_coefficient: f32,
    pub rolling_resistance_coefficient: f32,
    pub longitudinal_stiffness_per_slip: f32,
    pub lateral_stiffness_per_rad: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct GroundSurfaceCatalog {
    pub default: GroundSurfaceParams,
    pub grass: GroundSurfaceParams,
    pub mud: GroundSurfaceParams,
    pub rock: GroundSurfaceParams,
    pub asphalt: GroundSurfaceParams,
}

impl GroundSurfaceCatalog {
    pub fn params_for(&self, kind: GroundSurfaceKind) -> GroundSurfaceParams {
        match kind {
            GroundSurfaceKind::Default => self.default,
            GroundSurfaceKind::Grass => self.grass,
            GroundSurfaceKind::Mud => self.mud,
            GroundSurfaceKind::Rock => self.rock,
            GroundSurfaceKind::Asphalt => self.asphalt,
        }
    }
}

impl Default for GroundSurfaceCatalog {
    fn default() -> Self {
        Self {
            default: GroundSurfaceParams {
                friction_coefficient: 0.85,
                rolling_resistance_coefficient: 0.05,
                longitudinal_stiffness_per_slip: 7.0,
                lateral_stiffness_per_rad: 4.2,
            },
            grass: GroundSurfaceParams {
                friction_coefficient: 0.82,
                rolling_resistance_coefficient: 0.06,
                longitudinal_stiffness_per_slip: 6.5,
                lateral_stiffness_per_rad: 3.9,
            },
            mud: GroundSurfaceParams {
                friction_coefficient: 0.62,
                rolling_resistance_coefficient: 0.11,
                longitudinal_stiffness_per_slip: 4.5,
                lateral_stiffness_per_rad: 2.7,
            },
            rock: GroundSurfaceParams {
                friction_coefficient: 0.92,
                rolling_resistance_coefficient: 0.04,
                longitudinal_stiffness_per_slip: 8.0,
                lateral_stiffness_per_rad: 4.8,
            },
            asphalt: GroundSurfaceParams {
                friction_coefficient: 1.0,
                rolling_resistance_coefficient: 0.03,
                longitudinal_stiffness_per_slip: 8.8,
                lateral_stiffness_per_rad: 5.2,
            },
        }
    }
}
