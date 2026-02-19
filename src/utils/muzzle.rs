use bevy::{camera::primitives::MeshAabb, prelude::*};

pub fn compute_muzzle(mesh: &Mesh, forward_padding: f32) -> Option<Vec3> {
    let aabb = mesh.compute_aabb()?;

    let center: Vec3 = aabb.center.into();
    let half: Vec3 = aabb.half_extents.into();

    let forward_local = -Vec3::Z;
    let front_extent = half.dot(forward_local.abs());

    let gap: f32 = 0.02;

    let muzzle_offset = center + forward_local * (front_extent + forward_padding + gap);

    return Some(muzzle_offset);
}
