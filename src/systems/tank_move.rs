use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{ExternalForce, QueryFilter, ReadRapierContext, Velocity};

use crate::{
    components::{
        ground_surface::GroundSurfaceTag,
        intent::PlayerIntent,
        player::{Player, PlayerControllerState},
        tank::{GroundContact, TankHull, TankSuspension, TrackSide},
    },
    resources::{
        ground_surface_catalog::{GroundSurfaceCatalog, GroundSurfaceParams},
        player_physics_settings::PlayerPhysicsSettings,
    },
};

#[derive(Clone, Copy, Debug)]
struct SuspensionContactSample {
    side: TrackSide,
    world_point: Vec3,
    normal_load: f32,
    surface_params: GroundSurfaceParams,
    forward_tangent: Vec3,
    lateral_tangent: Vec3,
    forward_speed: f32,
    lateral_speed: f32,
}

fn point_velocity_at(velocity: &Velocity, world_com: Vec3, point: Vec3) -> Vec3 {
    velocity.linvel + velocity.angvel.cross(point - world_com)
}

fn project_onto_contact_plane(direction: Vec3, normal: Vec3) -> Vec3 {
    (direction - normal * direction.dot(normal)).normalize_or_zero()
}

fn lerp_linear(x0: f32, y0: f32, x1: f32, y1: f32, x: f32) -> f32 {
    if (x1 - x0).abs() <= f32::EPSILON {
        return y0;
    }
    let t = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
    y0 + (y1 - y0) * t
}

fn engine_torque_at_rpm(rpm: f32, settings: &PlayerPhysicsSettings) -> f32 {
    let idle_rpm = settings.engine_idle_rpm.max(100.0);
    let peak_rpm = settings.engine_peak_torque_rpm.max(idle_rpm + 1.0);
    let redline_rpm = settings.engine_redline_rpm.max(peak_rpm + 1.0);
    let clamped_rpm = rpm.clamp(idle_rpm, redline_rpm);

    if clamped_rpm <= peak_rpm {
        lerp_linear(
            idle_rpm,
            settings.engine_torque_idle_nm.max(0.0),
            peak_rpm,
            settings.engine_torque_peak_nm.max(0.0),
            clamped_rpm,
        )
    } else {
        lerp_linear(
            peak_rpm,
            settings.engine_torque_peak_nm.max(0.0),
            redline_rpm,
            settings.engine_torque_redline_nm.max(0.0),
            clamped_rpm,
        )
    }
}

fn engine_rpm_from_track_omega(
    mean_track_omega_abs: f32,
    gear_ratio: f32,
    settings: &PlayerPhysicsSettings,
) -> f32 {
    let idle_rpm = settings.engine_idle_rpm.max(100.0);
    let redline_rpm = settings.engine_redline_rpm.max(idle_rpm + 1.0);
    let final_drive = settings.final_drive_ratio.max(0.01);
    let engine_omega = mean_track_omega_abs.max(0.0) * gear_ratio.max(0.01) * final_drive;
    let rpm = engine_omega * 60.0 / TAU;
    rpm.clamp(idle_rpm, redline_rpm)
}

fn engine_rpm_from_ground_speed(
    ground_speed_abs: f32,
    gear_ratio: f32,
    settings: &PlayerPhysicsSettings,
) -> f32 {
    let sprocket_omega = ground_speed_abs.max(0.0) / settings.drive_sprocket_radius_m.max(0.05);
    engine_rpm_from_track_omega(sprocket_omega, gear_ratio, settings)
}

fn select_forward_gear_for_traction(
    ground_speed_abs: f32,
    settings: &PlayerPhysicsSettings,
) -> (usize, f32, f32) {
    let mut best_idx = 0usize;
    let mut best_ratio = settings.forward_gear_ratios[0].max(0.01);
    let mut best_rpm = engine_rpm_from_ground_speed(ground_speed_abs, best_ratio, settings);
    let mut best_force = f32::NEG_INFINITY;
    let final_drive = settings.final_drive_ratio.max(0.01);
    let efficiency = settings.drivetrain_efficiency.clamp(0.0, 1.0);
    let sprocket_radius = settings.drive_sprocket_radius_m.max(0.05);

    for (idx, ratio_raw) in settings.forward_gear_ratios.iter().enumerate() {
        let ratio = ratio_raw.max(0.01);
        let rpm = engine_rpm_from_ground_speed(ground_speed_abs, ratio, settings);
        let torque = engine_torque_at_rpm(rpm, settings);
        let force = torque * ratio * final_drive * efficiency / sprocket_radius;
        if force > best_force {
            best_force = force;
            best_idx = idx;
            best_ratio = ratio;
            best_rpm = rpm;
        }
    }

    (best_idx, best_ratio, best_rpm)
}

pub fn tank_hull_move_system(
    rapier_context: ReadRapierContext,
    time: Res<Time>,
    physics_settings: Res<PlayerPhysicsSettings>,
    ground_surface_catalog: Res<GroundSurfaceCatalog>,
    surface_tag_q: Query<&GroundSurfaceTag>,
    mut player_q: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &mut ExternalForce,
            &mut PlayerControllerState,
            &PlayerIntent,
            &TankSuspension,
            &mut GroundContact,
        ),
        (With<Player>, With<TankHull>),
    >,
) {
    let dt = time.delta_secs();
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    for (
        player_entity,
        player_tf,
        velocity,
        mut external_force,
        mut state,
        intent,
        suspension,
        mut ground_contact,
    ) in &mut player_q
    {
        let throttle_axis = intent.throttle.clamp(-1.0, 1.0);
        let turn_axis = intent.turn.clamp(-1.0, 1.0);
        let driver_throttle = throttle_axis.abs().clamp(0.0, 1.0);

        let hull_up = (player_tf.rotation * Vec3::Y).normalize_or_zero();
        let hull_forward = (player_tf.rotation * -Vec3::Z).normalize_or_zero();
        let world_com = player_tf.translation
            + player_tf.rotation * physics_settings.hull_center_of_mass_offset;

        let spring_axis = -hull_up;
        let rest_length = physics_settings.suspension_rest_length_m.max(0.01);
        let max_suspension_length = rest_length + physics_settings.suspension_travel_m.max(0.0);
        let fallback_surface_params = GroundSurfaceParams {
            friction_coefficient: physics_settings.track_friction_coefficient.max(0.0),
            rolling_resistance_coefficient: physics_settings
                .rolling_resistance_coefficient
                .max(0.0),
            longitudinal_stiffness_per_slip: physics_settings
                .track_longitudinal_stiffness_per_slip
                .max(0.0),
            lateral_stiffness_per_rad: physics_settings.track_lateral_stiffness_per_rad.max(0.0),
        };

        let mut total_force = Vec3::ZERO;
        let mut total_torque = Vec3::ZERO;
        let mut contacts = Vec::with_capacity(suspension.points.len());
        let mut weighted_normal_sum = Vec3::ZERO;
        let mut total_normal_load = 0.0_f32;

        for point in &suspension.points {
            let mount_world = player_tf.translation + player_tf.rotation * point.local_anchor;
            let Some((hit_entity, hit)) = rapier_context.cast_ray_and_get_normal(
                mount_world,
                spring_axis,
                max_suspension_length,
                true,
                QueryFilter::new()
                    .exclude_collider(player_entity)
                    .exclude_rigid_body(player_entity)
                    .exclude_sensors(),
            ) else {
                continue;
            };

            let contact_normal = hit.normal.normalize_or_zero();
            if contact_normal == Vec3::ZERO {
                continue;
            }

            let suspension_length = hit.time_of_impact.max(0.0);
            let compression = (rest_length - suspension_length).max(0.0);
            if compression <= 0.0 {
                continue;
            }

            let mount_velocity = point_velocity_at(velocity, world_com, mount_world);
            let compression_speed = mount_velocity.dot(spring_axis);
            let spring_force = physics_settings.suspension_stiffness_n_per_m.max(0.0) * compression;
            let damper_force =
                physics_settings.suspension_damping_n_per_mps.max(0.0) * compression_speed;
            let suspension_force_magnitude = (spring_force + damper_force).max(0.0);
            if suspension_force_magnitude <= 0.0 {
                continue;
            }

            let suspension_force = -spring_axis * suspension_force_magnitude;
            let suspension_arm = mount_world - world_com;
            total_force += suspension_force;
            total_torque += suspension_arm.cross(suspension_force);

            let normal_alignment = contact_normal.dot(-spring_axis).max(0.0);
            let normal_load = suspension_force_magnitude * normal_alignment;
            if normal_load <= 0.0 {
                continue;
            }

            let contact_world = mount_world + spring_axis * suspension_length;
            let forward_tangent = project_onto_contact_plane(hull_forward, contact_normal);
            if forward_tangent == Vec3::ZERO {
                continue;
            }
            let lateral_tangent = contact_normal.cross(forward_tangent).normalize_or_zero();
            if lateral_tangent == Vec3::ZERO {
                continue;
            }

            let contact_velocity = point_velocity_at(velocity, world_com, contact_world);
            let surface_params = surface_tag_q
                .get(hit_entity)
                .map(|tag| ground_surface_catalog.params_for(tag.kind))
                .unwrap_or(fallback_surface_params);
            contacts.push(SuspensionContactSample {
                side: point.side,
                world_point: contact_world,
                normal_load,
                surface_params,
                forward_tangent,
                lateral_tangent,
                forward_speed: contact_velocity.dot(forward_tangent),
                lateral_speed: contact_velocity.dot(lateral_tangent),
            });
            weighted_normal_sum += contact_normal * normal_load;
            total_normal_load += normal_load;
        }

        if total_normal_load > 0.0 {
            ground_contact.grounded = true;
            ground_contact.normal = (weighted_normal_sum / total_normal_load).normalize_or_zero();
            if ground_contact.normal == Vec3::ZERO {
                ground_contact.normal = Vec3::Y;
            }
            ground_contact.slope_angle = ground_contact.normal.dot(Vec3::Y).clamp(-1.0, 1.0).acos();
        } else {
            ground_contact.grounded = false;
            ground_contact.normal = Vec3::Y;
            ground_contact.slope_angle = 0.0;
        }

        let left_command = (throttle_axis - turn_axis).clamp(-1.0, 1.0);
        let right_command = (throttle_axis + turn_axis).clamp(-1.0, 1.0);

        let mut ground_speed_forward_weighted_sum = 0.0_f32;
        let mut left_contact_load = 0.0_f32;
        let mut right_contact_load = 0.0_f32;
        for contact in &contacts {
            ground_speed_forward_weighted_sum += contact.forward_speed * contact.normal_load;
            match contact.side {
                TrackSide::Left => left_contact_load += contact.normal_load,
                TrackSide::Right => right_contact_load += contact.normal_load,
            }
        }
        let ground_speed_forward = if total_normal_load > 0.0 {
            ground_speed_forward_weighted_sum / total_normal_load
        } else {
            velocity.linvel.dot(hull_forward)
        };
        let ground_speed_abs = ground_speed_forward.abs();

        let mean_track_omega_abs =
            0.5 * (state.left_track_angular_speed.abs() + state.right_track_angular_speed.abs());
        let (gear_index, active_ratio) = if throttle_axis < -0.05 {
            (-1_i8, physics_settings.reverse_gear_ratio.abs().max(0.01))
        } else {
            let (forward_gear_idx, ratio, _) =
                select_forward_gear_for_traction(ground_speed_abs, &physics_settings);
            ((forward_gear_idx as i8) + 1, ratio)
        };
        let neutral_steer_active = driver_throttle <= 0.05 && turn_axis.abs() > 0.05;
        let (
            left_drive_command,
            right_drive_command,
            left_brake_demand,
            right_brake_demand,
            throttle_demand,
        ) = if neutral_steer_active {
            // Neutral steer: opposite track torques with zero net longitudinal command.
            (-turn_axis, turn_axis, 0.0, 0.0, turn_axis.abs())
        } else {
            let gear_direction = if gear_index < 0 { -1.0 } else { 1.0 };
            let left_propulsion = (left_command * gear_direction).max(0.0);
            let right_propulsion = (right_command * gear_direction).max(0.0);
            let left_brake = (-left_command * gear_direction).max(0.0);
            let right_brake = (-right_command * gear_direction).max(0.0);
            (
                gear_direction * left_propulsion,
                gear_direction * right_propulsion,
                left_brake,
                right_brake,
                driver_throttle,
            )
        };

        let driveline_ratio = active_ratio * physics_settings.final_drive_ratio.max(0.01);
        let idle_omega = physics_settings.engine_idle_rpm.max(100.0) * TAU / 60.0;
        let mut engine_omega = if state.engine_rpm > 1.0 {
            state.engine_rpm * TAU / 60.0
        } else {
            idle_omega
        };
        let engine_rpm_from_state = (engine_omega * 60.0 / TAU).max(0.0);
        let peak_engine_torque = engine_torque_at_rpm(engine_rpm_from_state, &physics_settings);
        let governor_torque = (idle_omega - engine_omega).max(0.0)
            * physics_settings
                .engine_idle_governor_gain_nm_per_radps
                .max(0.0);
        let combustion_torque = peak_engine_torque * throttle_demand + governor_torque;

        let transmission_side_omega = mean_track_omega_abs * driveline_ratio.max(0.01);
        let clutch_slip = engine_omega - transmission_side_omega;
        let clutch_torque = (clutch_slip * physics_settings.clutch_coupling_nm_per_radps.max(0.0))
            .clamp(
                -physics_settings.clutch_max_torque_nm.max(0.0),
                physics_settings.clutch_max_torque_nm.max(0.0),
            );
        let engine_drag_torque =
            physics_settings.engine_viscous_drag_nm_per_radps.max(0.0) * engine_omega;
        let engine_inertia = physics_settings.engine_rotational_inertia_kg_m2.max(0.01);
        engine_omega +=
            (combustion_torque - clutch_torque - engine_drag_torque) / engine_inertia * dt;
        engine_omega = engine_omega.max(0.0);
        let engine_rpm =
            (engine_omega * 60.0 / TAU).clamp(0.0, physics_settings.engine_redline_rpm);

        let driveline_torque_budget = clutch_torque.abs()
            * driveline_ratio
            * physics_settings.drivetrain_efficiency.clamp(0.0, 1.0);
        let (left_drive_torque, right_drive_torque) = if throttle_demand > f32::EPSILON {
            (
                0.5 * driveline_torque_budget * left_drive_command,
                0.5 * driveline_torque_budget * right_drive_command,
            )
        } else {
            (0.0, 0.0)
        };

        let track_inertia = physics_settings.track_rotational_inertia_kg_m2.max(0.1);
        let left_omega_eval =
            state.left_track_angular_speed + left_drive_torque / track_inertia * dt;
        let right_omega_eval =
            state.right_track_angular_speed + right_drive_torque / track_inertia * dt;

        let slip_regularization_speed = physics_settings.slip_regularization_speed_mps.max(0.05);
        let sprocket_radius = physics_settings.drive_sprocket_radius_m.max(0.05);
        let mut left_ground_torque = 0.0_f32;
        let mut right_ground_torque = 0.0_f32;
        let mut left_slip_sum = 0.0_f32;
        let mut right_slip_sum = 0.0_f32;
        let mut mean_fx_sum = 0.0_f32;
        let mut mean_fy_sum = 0.0_f32;
        let mut contact_count = 0.0_f32;

        for contact in contacts {
            let mu = contact.surface_params.friction_coefficient.max(0.0);
            let rolling_resistance = contact
                .surface_params
                .rolling_resistance_coefficient
                .max(0.0);
            let longitudinal_slip_gain = contact
                .surface_params
                .longitudinal_stiffness_per_slip
                .max(0.0);
            let lateral_slip_gain = contact.surface_params.lateral_stiffness_per_rad.max(0.0);
            let track_omega = match contact.side {
                TrackSide::Left => left_omega_eval,
                TrackSide::Right => right_omega_eval,
            };
            let track_surface_speed = track_omega * sprocket_radius;
            let slip_denom = track_surface_speed
                .abs()
                .max(contact.forward_speed.abs())
                .max(slip_regularization_speed);
            let slip_ratio = (track_surface_speed - contact.forward_speed) / slip_denom;
            let traction_force =
                mu * contact.normal_load * (longitudinal_slip_gain * slip_ratio).tanh();
            let rolling_force =
                -contact.forward_speed.signum() * rolling_resistance * contact.normal_load;
            let desired_longitudinal = traction_force + rolling_force;

            let slip_angle = contact
                .lateral_speed
                .atan2(contact.forward_speed.abs() + slip_regularization_speed);
            let desired_lateral =
                -mu * contact.normal_load * (lateral_slip_gain * slip_angle).tanh();

            let contact_friction_limit = mu * contact.normal_load;
            let force_mag = Vec2::new(desired_longitudinal, desired_lateral).length();
            let force_scale = if force_mag > contact_friction_limit && force_mag > f32::EPSILON {
                contact_friction_limit / force_mag
            } else {
                1.0
            };
            let longitudinal_force = desired_longitudinal * force_scale;
            let lateral_force = desired_lateral * force_scale;

            match contact.side {
                TrackSide::Left => {
                    left_ground_torque += longitudinal_force * sprocket_radius;
                    left_slip_sum += slip_ratio * contact.normal_load;
                }
                TrackSide::Right => {
                    right_ground_torque += longitudinal_force * sprocket_radius;
                    right_slip_sum += slip_ratio * contact.normal_load;
                }
            }
            mean_fx_sum += longitudinal_force;
            mean_fy_sum += lateral_force;
            contact_count += 1.0;

            let track_force = contact.forward_tangent * longitudinal_force
                + contact.lateral_tangent * lateral_force;
            let track_arm = contact.world_point - world_com;
            total_force += track_force;
            total_torque += track_arm.cross(track_force);
        }

        let viscous_drag = physics_settings
            .drivetrain_viscous_drag_nm_per_radps
            .max(0.0);
        let brake_torque_max = physics_settings.track_brake_torque_max_nm.max(0.0);
        let left_drag_torque = viscous_drag * state.left_track_angular_speed;
        let right_drag_torque = viscous_drag * state.right_track_angular_speed;
        let left_brake_torque = brake_torque_max * left_brake_demand * left_omega_eval.signum();
        let right_brake_torque = brake_torque_max * right_brake_demand * right_omega_eval.signum();
        state.left_track_angular_speed +=
            (left_drive_torque - left_ground_torque - left_drag_torque - left_brake_torque)
                / track_inertia
                * dt;
        state.right_track_angular_speed +=
            (right_drive_torque - right_ground_torque - right_drag_torque - right_brake_torque)
                / track_inertia
                * dt;
        external_force.force = total_force;
        external_force.torque = total_torque;
        state.drive_velocity = velocity.linvel.dot(hull_forward);
        state.yaw_velocity = velocity.angvel.dot(hull_up);
        state.vertical_velocity = velocity.linvel.y;
        state.engine_rpm = engine_rpm;
        state.transmission_gear = gear_index;
        state.ground_speed_forward = ground_speed_forward;
        state.left_track_slip_ratio = if left_contact_load > 0.0 {
            left_slip_sum / left_contact_load
        } else {
            0.0
        };
        state.right_track_slip_ratio = if right_contact_load > 0.0 {
            right_slip_sum / right_contact_load
        } else {
            0.0
        };
        state.mean_contact_fx = if contact_count > 0.0 {
            mean_fx_sum / contact_count
        } else {
            0.0
        };
        state.mean_contact_fy = if contact_count > 0.0 {
            mean_fy_sum / contact_count
        } else {
            0.0
        };
    }
}
