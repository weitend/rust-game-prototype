use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LocalPlayerContext {
    pub entity: Option<Entity>,
}
