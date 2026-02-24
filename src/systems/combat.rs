use bevy::prelude::*;

use crate::{
    components::combat::{Health, Team},
    resources::combat_rules::{can_damage, CombatRules},
};

#[derive(Message, Clone, Copy, Debug)]
pub struct DamageEvent {
    pub source: Option<Entity>,
    pub target: Entity,
    pub amount: f32,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct DeathEvent {
    pub victim: Entity,
    pub killer: Option<Entity>,
}

pub fn apply_damage_system(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    rules: Res<CombatRules>,
    mut health_query: Query<(&mut Health, &Team)>,
    teams_query: Query<&Team>,
) {
    for event in damage_events.read() {
        if event.amount <= 0.0 {
            continue;
        }

        let Ok((mut health, target_team)) = health_query.get_mut(event.target) else {
            continue;
        };

        if health.current <= 0.0 {
            continue;
        }

        let attacker_team = event
            .source
            .and_then(|source| teams_query.get(source).ok())
            .copied();

        if !can_damage(attacker_team, *target_team, &rules) {
            continue;
        }

        health.current = (health.current - event.amount).clamp(0.0, health.max);

        if health.current == 0.0 {
            death_events.write(DeathEvent {
                victim: event.target,
                killer: event.source,
            });
        }
    }
}

pub fn handle_death_system(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    health_query: Query<(), With<Health>>,
) {
    for event in death_events.read() {
        let _ = event.killer;

        if health_query.get(event.victim).is_ok() {
            commands.entity(event.victim).despawn_children().despawn();
        }
    }
}
