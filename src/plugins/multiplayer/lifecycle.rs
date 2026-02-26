use bevy::prelude::*;

use super::state::NetLifecycleMessage;

pub(super) fn log_lifecycle_messages(mut messages: MessageReader<NetLifecycleMessage>) {
    for message in messages.read() {
        match message {
            NetLifecycleMessage::ServerSessionConnected { session_id, addr } => {
                eprintln!(
                    "[net-server] client connected: session_id={} addr={}",
                    session_id, addr
                );
            }
            NetLifecycleMessage::ServerSessionDisconnected {
                session_id,
                addr,
                reason,
            } => {
                eprintln!(
                    "[net-server] client disconnected: session_id={} addr={} reason={}",
                    session_id, addr, reason
                );
            }
            NetLifecycleMessage::ClientConnected {
                session_id,
                server_addr,
            } => {
                eprintln!(
                    "[net-client] connected: session_id={} server={}",
                    session_id, server_addr
                );
            }
            NetLifecycleMessage::ClientDisconnected { reason } => {
                eprintln!("[net-client] disconnected: reason={}", reason);
            }
        }
    }
}
