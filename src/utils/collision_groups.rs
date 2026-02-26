use bevy_rapier3d::prelude::{CollisionGroups, Group};

pub const GROUP_WORLD: Group = Group::GROUP_1;
pub const GROUP_PLAYER: Group = Group::GROUP_2;
pub const GROUP_ENEMY: Group = Group::GROUP_3;
pub const GROUP_DEBRIS: Group = Group::GROUP_4;

pub fn player_collision_groups() -> CollisionGroups {
    CollisionGroups::new(GROUP_PLAYER, GROUP_WORLD | GROUP_ENEMY | GROUP_PLAYER)
}

pub fn enemy_collision_groups() -> CollisionGroups {
    // Disable enemy-vs-enemy physical contacts to avoid heavy pile-up cost.
    CollisionGroups::new(GROUP_ENEMY, GROUP_WORLD | GROUP_PLAYER)
}

pub fn debris_collision_groups() -> CollisionGroups {
    CollisionGroups::new(GROUP_DEBRIS, GROUP_WORLD)
}
