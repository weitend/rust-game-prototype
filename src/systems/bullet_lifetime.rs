use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use crate::components::bullet::{Bullet, BulletLifeTime};

pub fn bullet_lifetyme_system(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut BulletLifeTime), With<Bullet>>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
