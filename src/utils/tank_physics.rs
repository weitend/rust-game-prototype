use bevy::prelude::{Quat, Vec3};
use bevy_rapier3d::prelude::{AdditionalMassProperties, Collider, MassProperties};

use crate::components::tank::{SuspensionPoint, TankSuspension, TrackSide};
use crate::resources::player_physics_settings::TankSpec;

pub fn tank_collider(outer_half_extents: Vec3, spec: &TankSpec) -> Collider {
    let max_radius = (outer_half_extents.min_element() - 0.01).max(0.0);
    let round_radius = spec.collider_round_radius.clamp(0.0, max_radius);
    let inner_half_extents =
        (outer_half_extents - Vec3::splat(round_radius)).max(Vec3::splat(0.01));

    Collider::round_cuboid(
        inner_half_extents.x,
        inner_half_extents.y,
        inner_half_extents.z,
        round_radius,
    )
}

pub fn tank_additional_mass_properties(spec: &TankSpec) -> AdditionalMassProperties {
    let principal_inertia = Vec3::new(
        spec.hull_principal_inertia.x.max(0.01),
        spec.hull_principal_inertia.y.max(0.01),
        spec.hull_principal_inertia.z.max(0.01),
    );

    AdditionalMassProperties::MassProperties(MassProperties {
        local_center_of_mass: spec.hull_center_of_mass_offset,
        mass: spec.hull_mass_kg.max(1.0),
        principal_inertia_local_frame: Quat::IDENTITY,
        principal_inertia,
    })
}

pub fn tank_suspension(outer_half_extents: Vec3) -> TankSuspension {
    let x = (outer_half_extents.x - 0.05).max(0.05);
    let y = -outer_half_extents.y + 0.06;
    let z = outer_half_extents.z;
    let z_points = [-0.84_f32, -0.42, 0.0, 0.42, 0.84];

    let mut points = Vec::with_capacity(z_points.len() * 2);
    for z_factor in z_points {
        let z_pos = z * z_factor;
        points.push(SuspensionPoint {
            local_anchor: Vec3::new(-x, y, z_pos),
            side: TrackSide::Left,
        });
        points.push(SuspensionPoint {
            local_anchor: Vec3::new(x, y, z_pos),
            side: TrackSide::Right,
        });
    }

    TankSuspension { points }
}
