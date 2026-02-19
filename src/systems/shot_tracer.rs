use bevy::prelude::*;

use crate::components::shot_tracer::{ShotTracer, ShotTracerLifetime};

pub fn update_shot_tracer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tracers: Query<(Entity, &ShotTracer, &mut Transform, &mut ShotTracerLifetime)>,
) {
    let dt = time.delta_secs();

    for (entity, tracer, mut transform, mut lifetime) in &mut tracers {
        transform.translation += tracer.velocity * dt;

        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
