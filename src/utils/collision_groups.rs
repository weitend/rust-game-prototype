use bevy_rapier3d::prelude::{CollisionGroups, Group};

pub const GROUP_WORLD: Group = Group::GROUP_1;
pub const GROUP_PLAYER: Group = Group::GROUP_2;
pub const GROUP_BULLET: Group = Group::GROUP_3;

pub fn player_collision_groups() -> CollisionGroups {
    CollisionGroups::new(GROUP_PLAYER, GROUP_WORLD)
}

pub fn bullet_collision_groups() -> CollisionGroups {
    CollisionGroups::new(GROUP_BULLET, GROUP_WORLD)
}
