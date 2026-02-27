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

pub fn muzzle_ray_from_local_hierarchy(
    hull_tf: &Transform,
    turret_tf: &Transform,
    barrel_tf: &Transform,
    muzzle_tf: &Transform,
) -> Option<(Vec3, Vec3)> {
    let world =
        hull_tf.to_matrix() * turret_tf.to_matrix() * barrel_tf.to_matrix() * muzzle_tf.to_matrix();
    let ray_origin = world.transform_point3(Vec3::ZERO);
    let ray_dir = world.transform_vector3(-Vec3::Z).normalize_or_zero();

    if ray_dir == Vec3::ZERO {
        None
    } else {
        Some((ray_origin, ray_dir))
    }
}
