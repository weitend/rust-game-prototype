use bevy::prelude::*;

use crate::{
    components::combat::Health,
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
) {
    for impact in impact_events.read() {
        route_damage(impact, &damageable_targets, &mut damage_events);
    }
}

fn route_damage(
    impact: &ImpactEvent,
    damageable_targets: &Query<(), With<Health>>,
    damage_events: &mut MessageWriter<DamageEvent>,
) {
    if damageable_targets.contains(impact.target) {
        damage_events.write(DamageEvent {
            source: impact.source,
            target: impact.target,
            amount: impact.damage,
        });
    }
}

