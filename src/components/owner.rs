use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedBy {
    pub entity: Entity,
}
