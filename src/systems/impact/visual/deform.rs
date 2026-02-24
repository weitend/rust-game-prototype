use bevy::{math::Affine3A, mesh::VertexAttributeValues, prelude::*};

use crate::{
    components::destructible_mesh::DestructibleMesh,
    resources::impact_assets::ImpactAssets,
    utils::impact_math::normalized_or_up,
};

use super::sampling::ImpactSample;

pub(super) fn apply_dents_to_obstacle_mesh(
    meshes: &mut Assets<Mesh>,
    mesh_handle: &Handle<Mesh>,
    impact_assets: &ImpactAssets,
    samples: &[ImpactSample],
    impact_normal: Vec3,
    world_to_local: &Affine3A,
    dents_to_apply: usize,
    deformable_mesh: &DestructibleMesh,
) -> usize {
    if dents_to_apply == 0 {
        return 0;
    }

    let Some(mesh) = meshes.get_mut(mesh_handle) else {
        return 0;
    };

    let local_normal = normalized_or_up(world_to_local.transform_vector3(impact_normal));
    let applied_dents = {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            return 0;
        };

        let mut applied = 0usize;
        for sample in samples.iter().take(dents_to_apply) {
            let local_center = world_to_local.transform_point3(sample.world_point);
            let radius =
                (impact_assets.crater_size * sample.size_scale).max(impact_assets.base_web_radius);
            let depth = (impact_assets.crater_depth * sample.depth_scale)
                .min(deformable_mesh.max_dent_depth)
                .max(0.001);
            let normal_band = (radius * 0.45).max(0.01);

            let mut touched_any_vertex = false;
            for position in positions.iter_mut() {
                let vertex = Vec3::from_array(*position);
                let delta = vertex - local_center;
                let plane_distance = delta.dot(local_normal);
                if plane_distance.abs() > normal_band {
                    continue;
                }

                let tangent = delta - local_normal * plane_distance;
                let tangent_distance = tangent.length();
                if tangent_distance > radius {
                    continue;
                }

                let radial_falloff = 1.0 - tangent_distance / radius;
                let band_falloff = 1.0 - (plane_distance.abs() / normal_band);
                let requested = depth * radial_falloff.powi(2) * band_falloff.powi(2);
                let clamped_plane =
                    (plane_distance - requested).max(-deformable_mesh.max_dent_depth);
                let applied_depth = plane_distance - clamped_plane;
                if applied_depth <= f32::EPSILON {
                    continue;
                }

                let deformed = vertex - local_normal * applied_depth;
                *position = deformed.to_array();
                touched_any_vertex = true;
            }

            if touched_any_vertex {
                applied += 1;
            }
        }
        applied
    };

    if applied_dents > 0 {
        mesh.compute_smooth_normals();
        mesh.remove_attribute(Mesh::ATTRIBUTE_TANGENT);
    }

    applied_dents
}

