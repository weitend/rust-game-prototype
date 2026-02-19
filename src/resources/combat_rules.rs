use bevy::prelude::Resource;

use crate::components::combat::Team;

#[derive(Resource, Clone, Copy, Debug)]
pub struct CombatRules {
    pub friendly_fire: bool,
}

impl Default for CombatRules {
    fn default() -> Self {
        Self {
            friendly_fire: false,
        }
    }
}

pub fn can_damage(attacker_team: Option<Team>, target_team: Team, rules: &CombatRules) -> bool {
    match attacker_team {
        Some(attacker_team) if attacker_team == target_team => rules.friendly_fire,
        _ => true,
    }
}
