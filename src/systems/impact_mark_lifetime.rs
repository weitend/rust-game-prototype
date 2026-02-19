use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use crate::components::impact_mark_lifetime::ImpactMarkLifetime;

pub fn impact_mark_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut marks: Query<(Entity, &mut ImpactMarkLifetime)>,
) {
    for (entity, mut lifetime) in &mut marks {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
