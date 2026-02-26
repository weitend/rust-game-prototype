use bevy::prelude::*;

use crate::components::{combat::Health, owner::OwnedBy};

pub fn resolve_damage_target(
    target: Entity,
    damageable_targets: &Query<(), With<Health>>,
    owned_targets: &Query<&OwnedBy>,
) -> Option<Entity> {
    if damageable_targets.contains(target) {
        return Some(target);
    }

    owned_targets
        .get(target)
        .ok()
        .and_then(|owner| damageable_targets.contains(owner.entity).then_some(owner.entity))
}
