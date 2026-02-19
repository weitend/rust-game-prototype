use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{bullet::Bullet, impact_mark_lifetime::ImpactMarkLifetime, obstacle::Obstacle},
    resources::bullet_assets::BulletAssets,
};

const IMPACT_MARK_LIFETIME_SECS: f32 = 30.0;

pub fn bullet_hit_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    bullets: Query<&Transform, With<Bullet>>,
    obstacles: Query<(), With<Obstacle>>,
    rapier_context: ReadRapierContext,
    bullet_assets: Res<BulletAssets>,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    let mut processed_bullets: HashSet<Entity> = HashSet::new();

    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            continue;
        };

        let (bullet_entity, obstacle_entity) = if bullets.contains(e1) && obstacles.contains(e2) {
            (e1, e2)
        } else if bullets.contains(e2) && obstacles.contains(e1) {
            (e2, e1)
        } else {
            continue;
        };

        if !processed_bullets.insert(bullet_entity) {
            continue;
        }

        let mut mark_position = bullets.get(bullet_entity).ok().map(|tf| tf.translation);

        if let Some(pair) = rapier_context.contact_pair(bullet_entity, obstacle_entity) {
            if let Some(manifold) = pair.manifolds().next() {
                if let Some(contact) = manifold.solver_contacts().next() {
                    mark_position = Some(
                        contact.point() + manifold.normal() * (bullet_assets.impact_radius * 0.35),
                    )
                }
            }
        }

        if let Some(mark_position) = mark_position {
            commands.spawn((
                Mesh3d(bullet_assets.impact_mesh.clone()),
                MeshMaterial3d(bullet_assets.impact_material.clone()),
                Transform::from_translation(mark_position),
                ImpactMarkLifetime {
                    timer: Timer::from_seconds(IMPACT_MARK_LIFETIME_SECS, TimerMode::Once),
                },
            ));
        }

        commands.entity(bullet_entity).despawn();
    }
}
