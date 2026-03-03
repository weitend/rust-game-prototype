use bevy::prelude::*;

use crate::{
    components::{
        owner::OwnedBy,
        tank::{TrackLinkVisual, TrackSide},
    },
    resources::player_spawn::PlayerTemplate,
    systems::track_visual::{track_link_count, track_loop_length_m, track_pose_from_phase},
};

pub const TURRET_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.46, 0.0);
pub const BARREL_PIVOT_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.09, -0.44);
pub const BARREL_VISUAL_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.0, -0.63);
pub const MUZZLE_LOCAL_OFFSET: Vec3 = Vec3::new(0.0, 0.0, -1.26);

pub fn spawn_track_visuals_for_entity(
    commands: &mut Commands,
    template: &PlayerTemplate,
    parent_entity: Entity,
    link_owner: Option<Entity>,
) {
    let link_count = track_link_count();
    let loop_length_m = track_loop_length_m();
    let guide_inset_x = 0.02_f32;

    for guide_idx in 0..link_count {
        let phase = (guide_idx as f32 / link_count as f32) * loop_length_m;
        for side in [TrackSide::Left, TrackSide::Right] {
            let (mut guide_position, guide_rotation) = track_pose_from_phase(side, phase);
            guide_position.x += match side {
                TrackSide::Left => guide_inset_x,
                TrackSide::Right => -guide_inset_x,
            };
            let name = match side {
                TrackSide::Left => "TankTrack::LeftGuide",
                TrackSide::Right => "TankTrack::RightGuide",
            };
            let guide = commands
                .spawn((
                    Name::new(name),
                    Mesh3d(template.track_belt_mesh.clone()),
                    MeshMaterial3d(template.track_belt_material.clone()),
                    Transform::from_translation(guide_position).with_rotation(guide_rotation),
                ))
                .id();
            commands.entity(parent_entity).add_child(guide);
        }
    }

    for link_idx in 0..link_count {
        let phase = (link_idx as f32 / link_count as f32) * loop_length_m;
        let (left_position, left_rotation) = track_pose_from_phase(TrackSide::Left, phase);
        let left_link = if let Some(owner_entity) = link_owner {
            commands
                .spawn((
                    Name::new("TankTrack::LeftLink"),
                    Mesh3d(template.track_link_mesh.clone()),
                    MeshMaterial3d(template.track_link_material.clone()),
                    Transform::from_translation(left_position).with_rotation(left_rotation),
                    OwnedBy {
                        entity: owner_entity,
                    },
                    TrackLinkVisual {
                        side: TrackSide::Left,
                        base_phase_m: phase,
                    },
                ))
                .id()
        } else {
            commands
                .spawn((
                    Name::new("TankTrack::LeftLink"),
                    Mesh3d(template.track_link_mesh.clone()),
                    MeshMaterial3d(template.track_link_material.clone()),
                    Transform::from_translation(left_position).with_rotation(left_rotation),
                ))
                .id()
        };

        let (right_position, right_rotation) = track_pose_from_phase(TrackSide::Right, phase);
        let right_link = if let Some(owner_entity) = link_owner {
            commands
                .spawn((
                    Name::new("TankTrack::RightLink"),
                    Mesh3d(template.track_link_mesh.clone()),
                    MeshMaterial3d(template.track_link_material.clone()),
                    Transform::from_translation(right_position).with_rotation(right_rotation),
                    OwnedBy {
                        entity: owner_entity,
                    },
                    TrackLinkVisual {
                        side: TrackSide::Right,
                        base_phase_m: phase,
                    },
                ))
                .id()
        } else {
            commands
                .spawn((
                    Name::new("TankTrack::RightLink"),
                    Mesh3d(template.track_link_mesh.clone()),
                    MeshMaterial3d(template.track_link_material.clone()),
                    Transform::from_translation(right_position).with_rotation(right_rotation),
                ))
                .id()
        };

        commands.entity(parent_entity).add_child(left_link);
        commands.entity(parent_entity).add_child(right_link);
    }
}
