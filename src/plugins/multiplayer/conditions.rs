use bevy::prelude::*;

use crate::resources::run_mode::{AppRunMode, RunMode};

pub(super) fn is_server_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Server | RunMode::Host)
}

pub(super) fn is_client_like_mode(mode: Res<AppRunMode>) -> bool {
    matches!(mode.0, RunMode::Client | RunMode::Host)
}
