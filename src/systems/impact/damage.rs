use bevy::prelude::*;

use crate::{
    components::{combat::Health, owner::OwnedBy},
    systems::combat::DamageEvent,
};

#[derive(Message, Clone, Copy, Debug)]
pub struct ImpactEvent {
    pub source: Option<Entity>,
    pub target: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub damage: f32,
}

pub fn route_impact_damage_system(
    mut impact_events: MessageReader<ImpactEvent>,
    mut damage_events: MessageWriter<DamageEvent>,
    damageable_targets: Query<(), With<Health>>,
    owned_targets: Query<&OwnedBy>,
) {
    for impact in impact_events.read() {
        route_damage(impact, &damageable_targets, &owned_targets, &mut damage_events);
    }
}

fn route_damage(
    impact: &ImpactEvent,
    damageable_targets: &Query<(), With<Health>>,
    owned_targets: &Query<&OwnedBy>,
    damage_events: &mut MessageWriter<DamageEvent>,
) {
    let resolved_target = if damageable_targets.contains(impact.target) {
        Some(impact.target)
    } else if let Ok(owner) = owned_targets.get(impact.target) {
        damageable_targets
            .contains(owner.entity)
            .then_some(owner.entity)
    } else {
        None
    };

    let Some(target) = resolved_target else {
        return;
    };

    damage_events.write(DamageEvent {
        source: impact.source,
        target,
        amount: impact.damage,
    });
}
