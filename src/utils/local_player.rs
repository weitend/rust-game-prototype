use bevy::prelude::*;

use crate::{
    components::player::{LocalPlayer, Player},
    resources::local_player::LocalPlayerContext,
};

pub fn resolve_local_player_entity(
    ctx: &LocalPlayerContext,
    local_player_q: &Query<Entity, (With<Player>, With<LocalPlayer>)>,
) -> Option<Entity> {
    if let Some(entity) = ctx.entity {
        if local_player_q.get(entity).is_ok() {
            return Some(entity);
        }
    }

    let mut local_players = local_player_q.iter();
    let entity = local_players.next()?;
    if local_players.next().is_some() {
        return None;
    }

    Some(entity)
}
