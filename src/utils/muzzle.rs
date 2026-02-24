use bevy::{camera::primitives::MeshAabb, prelude::*};

pub fn compute_muzzle(mesh: &Mesh, forward_padding: f32) -> Option<Vec3> {
    let aabb = mesh.compute_aabb()?;

    let center: Vec3 = aabb.center.into();
    let half: Vec3 = aabb.half_extents.into();

    let forward_local = -Vec3::Z;
    let front_extent = half.dot(forward_local.abs());

    let gap: f32 = 0.02;

    let muzzle_offset = center + forward_local * (front_extent + forward_padding + gap);

    Some(muzzle_offset)
}

pub fn muzzle_ray(muzzle_tf: &GlobalTransform) -> Option<(Vec3, Vec3)> {
    let ray_origin = muzzle_tf.translation();
    let (_, muzzle_rotation, _) = muzzle_tf.to_scale_rotation_translation();
    let ray_dir = (muzzle_rotation * -Vec3::Z).normalize_or_zero();

    if ray_dir == Vec3::ZERO {
        None
    } else {
        Some((ray_origin, ray_dir))
    }
}
