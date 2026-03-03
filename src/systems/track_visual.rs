use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{prelude::*, time::Fixed};

use crate::{
    components::{
        owner::OwnedBy,
        player::{Player, PlayerControllerState},
        tank::{TrackLinkVisual, TrackSide, TrackVisualPhase},
    },
    resources::player_physics_settings::PlayerPhysicsSettings,
};

pub const TRACK_SIDE_OFFSET_X_M: f32 = 0.93;
pub const TRACK_FRONT_Z_M: f32 = -0.84;
pub const TRACK_REAR_Z_M: f32 = 0.84;
pub const TRACK_WHEEL_CENTER_Y_M: f32 = -0.53;
pub const TRACK_WHEEL_RADIUS_M: f32 = 0.20;
pub const TRACK_LINK_PITCH_M: f32 = 0.06;

pub fn track_loop_length_m() -> f32 {
    let straight = (TRACK_REAR_Z_M - TRACK_FRONT_Z_M).max(0.0);
    2.0 * straight + 2.0 * PI * TRACK_WHEEL_RADIUS_M
}

pub fn track_link_count() -> usize {
    let count = (track_loop_length_m() / TRACK_LINK_PITCH_M)
        .floor()
        .max(32.0) as usize;
    count.max(32)
}

pub fn track_pose_from_phase(side: TrackSide, phase_m: f32) -> (Vec3, Quat) {
    let straight_len = (TRACK_REAR_Z_M - TRACK_FRONT_Z_M).max(0.0);
    let arc_len = PI * TRACK_WHEEL_RADIUS_M;
    let loop_len = track_loop_length_m();
    let mut s = phase_m.rem_euclid(loop_len);

    let (z, y, tan_z, tan_y) = if s <= straight_len {
        let z = TRACK_FRONT_Z_M + s;
        (z, TRACK_WHEEL_CENTER_Y_M + TRACK_WHEEL_RADIUS_M, 1.0, 0.0)
    } else if s <= straight_len + arc_len {
        s -= straight_len;
        let theta = FRAC_PI_2 - s / TRACK_WHEEL_RADIUS_M;
        let z = TRACK_REAR_Z_M + TRACK_WHEEL_RADIUS_M * theta.cos();
        let y = TRACK_WHEEL_CENTER_Y_M + TRACK_WHEEL_RADIUS_M * theta.sin();
        let tan_z = theta.sin();
        let tan_y = -theta.cos();
        (z, y, tan_z, tan_y)
    } else if s <= 2.0 * straight_len + arc_len {
        s -= straight_len + arc_len;
        let z = TRACK_REAR_Z_M - s;
        (z, TRACK_WHEEL_CENTER_Y_M - TRACK_WHEEL_RADIUS_M, -1.0, 0.0)
    } else {
        s -= 2.0 * straight_len + arc_len;
        let theta = -FRAC_PI_2 + s / TRACK_WHEEL_RADIUS_M;
        let z = TRACK_FRONT_Z_M - TRACK_WHEEL_RADIUS_M * theta.cos();
        let y = TRACK_WHEEL_CENTER_Y_M + TRACK_WHEEL_RADIUS_M * theta.sin();
        let tan_z = theta.sin();
        let tan_y = theta.cos();
        (z, y, tan_z, tan_y)
    };

    let x = match side {
        TrackSide::Left => -TRACK_SIDE_OFFSET_X_M,
        TrackSide::Right => TRACK_SIDE_OFFSET_X_M,
    };
    let tangent = Vec3::new(0.0, tan_y, tan_z).normalize_or_zero();
    let rotation = if tangent == Vec3::ZERO {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_arc(Vec3::Z, tangent)
    };
    (Vec3::new(x, y, z), rotation)
}

fn lerp_phase(prev: f32, curr: f32, alpha: f32, loop_len: f32) -> f32 {
    let half = 0.5 * loop_len;
    let delta = (curr - prev + half).rem_euclid(loop_len) - half;
    (prev + delta * alpha).rem_euclid(loop_len)
}

pub fn integrate_track_visual_phase_fixed_system(
    time: Res<Time>,
    physics_settings: Res<PlayerPhysicsSettings>,
    mut phase_q: Query<(&mut TrackVisualPhase, &PlayerControllerState), With<Player>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let sprocket_radius = physics_settings.drive_sprocket_radius_m.max(0.05);
    let loop_len = track_loop_length_m();
    for (mut phase, state) in &mut phase_q {
        phase.prev_left_m = phase.curr_left_m;
        phase.prev_right_m = phase.curr_right_m;

        phase.curr_left_m = (phase.curr_left_m
            + state.left_track_angular_speed * sprocket_radius * dt)
            .rem_euclid(loop_len);
        phase.curr_right_m = (phase.curr_right_m
            + state.right_track_angular_speed * sprocket_radius * dt)
            .rem_euclid(loop_len);
    }
}

pub fn animate_track_visuals_system(
    fixed_time: Res<Time<Fixed>>,
    phase_q: Query<&TrackVisualPhase, With<Player>>,
    mut link_q: Query<(&mut Transform, &TrackLinkVisual, &OwnedBy)>,
) {
    let alpha = fixed_time.overstep_fraction();
    let loop_len = track_loop_length_m();

    for (mut link_tf, link, owner) in &mut link_q {
        let Ok(phase) = phase_q.get(owner.entity) else {
            continue;
        };

        let side_phase = match link.side {
            TrackSide::Left => lerp_phase(phase.prev_left_m, phase.curr_left_m, alpha, loop_len),
            TrackSide::Right => lerp_phase(phase.prev_right_m, phase.curr_right_m, alpha, loop_len),
        };
        let world_phase = (link.base_phase_m + side_phase).rem_euclid(loop_len);
        let (position, rotation) = track_pose_from_phase(link.side, world_phase);
        link_tf.translation = position;
        link_tf.rotation = rotation;
    }
}
