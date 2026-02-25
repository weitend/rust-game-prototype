use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetEntityId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInput {
    pub seq: u32,
    pub throttle: f32,
    pub turn: f32,
    pub turret_yaw_delta: f32,
    pub barrel_pitch_delta: f32,
    pub fire_pressed: bool,
    pub fire_just_pressed: bool,
    pub artillery_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: NetEntityId,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerEventDto {
    SessionAnnounce { text: String },
    VehicleSpawned { id: NetEntityId },
    VehicleDespawned { id: NetEntityId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u32,
    pub entities: Vec<EntitySnapshot>,
    pub events: Vec<ServerEventDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    Hello { protocol_version: u16, nonce: u64 },
    Ping { seq: u32 },
    Input(ClientInput),
    Disconnect { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    Welcome {
        protocol_version: u16,
        session_id: u64,
    },
    Pong {
        seq: u32,
        server_tick: u32,
    },
    Snapshot(Snapshot),
    Event(ServerEventDto),
    Disconnect {
        reason: String,
    },
}
