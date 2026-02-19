use bevy::{ecs::component::Component, time::Timer};

#[derive(Component)]
pub struct ImpactMarkLifetime {
    pub timer: Timer,
}
