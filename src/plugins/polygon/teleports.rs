use bevy::{prelude::*, text::Justify};

use crate::components::player::{Player, PlayerControllerState};

const TELEPORT_LABEL_SHOW_DISTANCE: f32 = 6.5;

#[derive(Component, Clone, Copy, Debug)]
pub struct TeleportPad {
    pub destination: Vec3,
    pub half_extents: Vec2,
}

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct TeleportLabelAnchor {
    pad: Entity,
    world_offset: Vec3,
}

#[derive(Resource, Default)]
pub struct TeleportRuntime {
    cooldown_secs: f32,
}

pub fn spawn_teleport_pad(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    font: &Handle<Font>,
    position: Vec3,
    destination: Vec3,
    label: &str,
    color: Color,
) {
    let pad_size = Vec3::new(3.0, 0.24, 3.0);

    let pad_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(pad_size.x, pad_size.y, pad_size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.88,
                metallic: 0.0,
                ..default()
            })),
            Transform::from_translation(position),
            TeleportPad {
                destination,
                half_extents: Vec2::new(pad_size.x * 0.5, pad_size.z * 0.5),
            },
        ))
        .id();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            TeleportLabelAnchor {
                pad: pad_entity,
                world_offset: Vec3::new(0.0, 2.2, 0.0),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label.to_owned()),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.97, 0.98, 1.0)),
                TextLayout::new_with_justify(Justify::Center).with_no_wrap(),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(30.0),
                    left: px(-140.0),
                    width: px(280.0),
                    padding: UiRect::axes(px(8.0), px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.74)),
                BorderRadius::all(px(6.0)),
            ));
        });
}

pub fn teleport_player_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut runtime: ResMut<TeleportRuntime>,
    pads: Query<(&Transform, &TeleportPad), Without<Player>>,
    mut player_q: Query<(&mut Transform, &mut PlayerControllerState), With<Player>>,
) {
    runtime.cooldown_secs = (runtime.cooldown_secs - time.delta_secs()).max(0.0);

    if runtime.cooldown_secs > 0.0 || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok((mut player_transform, mut player_state)) = player_q.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation;
    let mut best_destination: Option<(Vec3, f32)> = None;

    for (pad_tf, pad) in &pads {
        let delta = player_pos - pad_tf.translation;
        let inside_x = delta.x.abs() <= pad.half_extents.x;
        let inside_z = delta.z.abs() <= pad.half_extents.y;
        let close_y = delta.y.abs() <= 2.4;

        if !(inside_x && inside_z && close_y) {
            continue;
        }

        let planar_dist_sq = Vec2::new(delta.x, delta.z).length_squared();

        match best_destination {
            Some((_, current_best)) if planar_dist_sq >= current_best => {}
            _ => best_destination = Some((pad.destination, planar_dist_sq)),
        }
    }

    if let Some((destination, _)) = best_destination {
        player_transform.translation = destination;
        player_state.vertical_velocity = 0.0;
        runtime.cooldown_secs = 0.35;
    }
}

pub fn sync_teleport_labels_system(
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_q: Query<&Transform, With<Player>>,
    pads: Query<&GlobalTransform, With<TeleportPad>>,
    mut labels: Query<(&TeleportLabelAnchor, &mut Node, &mut Visibility)>,
) {
    let Ok((camera, camera_tf)) = camera_q.single() else {
        return;
    };
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;
    let max_dist_sq = TELEPORT_LABEL_SHOW_DISTANCE * TELEPORT_LABEL_SHOW_DISTANCE;

    for (label_anchor, mut label_node, mut label_visibility) in &mut labels {
        let Ok(pad_tf) = pads.get(label_anchor.pad) else {
            *label_visibility = Visibility::Hidden;
            continue;
        };

        let pad_pos = pad_tf.translation();
        let planar_dist_sq =
            Vec2::new(player_pos.x - pad_pos.x, player_pos.z - pad_pos.z).length_squared();
        if planar_dist_sq > max_dist_sq {
            *label_visibility = Visibility::Hidden;
            continue;
        }

        let world_pos = pad_tf.translation() + label_anchor.world_offset;
        match camera.world_to_viewport(camera_tf, world_pos) {
            Ok(viewport) => {
                label_node.left = px(viewport.x);
                label_node.top = px(viewport.y);
                *label_visibility = Visibility::Visible;
            }
            Err(_) => {
                *label_visibility = Visibility::Hidden;
            }
        }
    }
}
