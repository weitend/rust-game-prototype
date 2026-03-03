use bevy::prelude::Component;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GroundSurfaceKind {
    Default,
    Grass,
    Mud,
    Rock,
    Asphalt,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct GroundSurfaceTag {
    pub kind: GroundSurfaceKind,
}

impl GroundSurfaceTag {
    pub const fn new(kind: GroundSurfaceKind) -> Self {
        Self { kind }
    }
}

impl Default for GroundSurfaceTag {
    fn default() -> Self {
        Self {
            kind: GroundSurfaceKind::Default,
        }
    }
}
