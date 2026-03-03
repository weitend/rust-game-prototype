use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{
    components::{
        player::{LocalPlayer, Player, PlayerControllerState},
        tank::GroundContact,
    },
    plugins::multiplayer::ClientNetworkTelemetry,
    resources::local_player::LocalPlayerContext,
    utils::local_player::resolve_local_player_entity,
};

#[derive(Resource, Clone, Debug, Default)]
pub struct MovementMetricsSnapshot {
    pub lines: Vec<String>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FpsMetricsSnapshot {
    pub fps_line: Option<String>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct NetworkMetricsSnapshot {
    pub ping_ms: Option<f32>,
    pub server_tick: Option<u32>,
    pub packet_loss_pct: Option<f32>,
    pub status: Option<String>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub movement_lines: Vec<String>,
    pub fps_line: Option<String>,
    pub network_lines: Vec<String>,
}

pub fn collect_movement_metrics_system(
    local_player_ctx: Res<LocalPlayerContext>,
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    state_q: Query<(&PlayerControllerState, Option<&GroundContact>), With<Player>>,
    mut snapshot: ResMut<MovementMetricsSnapshot>,
) {
    let Some(player_entity) = resolve_local_player_entity(&local_player_ctx, &local_player_q)
    else {
        snapshot.lines = vec!["move: no local player".to_owned()];
        return;
    };

    let Ok((state, contact)) = state_q.get(player_entity) else {
        snapshot.lines = vec!["move: no controller state".to_owned()];
        return;
    };

    let mut lines = Vec::with_capacity(10);
    lines.push(format!("v: {:.2} m/s", state.drive_velocity));
    lines.push(format!("yaw: {:.2} rad/s", state.yaw_velocity));
    lines.push(format!("rpm: {:.0}", state.engine_rpm));
    lines.push(format!("gear: {}", state.transmission_gear));
    lines.push(format!(
        "slip L/R: {:.2} / {:.2}",
        state.left_track_slip_ratio, state.right_track_slip_ratio
    ));
    lines.push(format!(
        "Fx/Fy: {:.0} / {:.0} N",
        state.mean_contact_fx, state.mean_contact_fy
    ));
    lines.push(format!(
        "omega L/R: {:.2} / {:.2} rad/s",
        state.left_track_angular_speed, state.right_track_angular_speed
    ));
    lines.push(format!("ground v: {:.2} m/s", state.ground_speed_forward));
    if let Some(contact) = contact {
        lines.push(format!(
            "grounded: {} slope: {:.1} deg",
            contact.grounded,
            contact.slope_angle.to_degrees()
        ));
    }
    snapshot.lines = lines;
}

pub fn collect_fps_metrics_system(
    diagnostics: Res<DiagnosticsStore>,
    mut snapshot: ResMut<FpsMetricsSnapshot>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed());
    snapshot.fps_line = fps.map(|value| format!("fps: {:.1}", value));
}

pub fn collect_network_metrics_system(
    telemetry: Option<Res<ClientNetworkTelemetry>>,
    mut snapshot: ResMut<NetworkMetricsSnapshot>,
) {
    let Some(telemetry) = telemetry else {
        snapshot.ping_ms = None;
        snapshot.server_tick = None;
        snapshot.packet_loss_pct = None;
        snapshot.status = Some("net: no data".to_owned());
        return;
    };

    snapshot.ping_ms = telemetry.ping_ms;
    snapshot.server_tick = telemetry.server_tick;
    snapshot.packet_loss_pct = telemetry.packet_loss_pct;
    snapshot.status = Some(telemetry.status.clone());
}

pub fn compose_metrics_snapshot_system(
    movement: Res<MovementMetricsSnapshot>,
    fps: Res<FpsMetricsSnapshot>,
    network: Res<NetworkMetricsSnapshot>,
    mut snapshot: ResMut<MetricsSnapshot>,
) {
    snapshot.movement_lines = movement.lines.clone();
    snapshot.fps_line = fps.fps_line.clone();

    let mut network_lines = Vec::with_capacity(4);
    network_lines.push(
        network
            .status
            .clone()
            .unwrap_or_else(|| "net: no data".to_owned()),
    );
    if let Some(ping_ms) = network.ping_ms {
        network_lines.push(format!("ping: {:.1} ms", ping_ms));
    }
    if let Some(server_tick) = network.server_tick {
        network_lines.push(format!("tick: {}", server_tick));
    }
    if let Some(packet_loss_pct) = network.packet_loss_pct {
        network_lines.push(format!("loss: {:.1}%", packet_loss_pct));
    }
    snapshot.network_lines = network_lines;
}
