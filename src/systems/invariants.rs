use bevy::prelude::*;

use crate::components::{
    owner::OwnedBy,
    player::{LocalPlayer, Player},
    tank::{TankBarrel, TankMuzzle, TankParts, TankTurret},
};

pub fn debug_validate_invariants_system(
    local_player_q: Query<Entity, (With<Player>, With<LocalPlayer>)>,
    players_q: Query<(Entity, &TankParts), With<Player>>,
    turret_q: Query<&OwnedBy, With<TankTurret>>,
    barrel_q: Query<&OwnedBy, With<TankBarrel>>,
    muzzle_q: Query<&OwnedBy, With<TankMuzzle>>,
) {
    let local_players = local_player_q.iter().count();
    match local_players {
        0 => warn!("Invariant violation: expected exactly one LocalPlayer, found 0"),
        1 => {}
        n => warn!(
            "Invariant violation: expected exactly one LocalPlayer, found {}",
            n
        ),
    }

    for (player_entity, tank_parts) in &players_q {
        validate_owned_part::<TankTurret>("turret", tank_parts.turret, player_entity, &turret_q);
        validate_owned_part::<TankBarrel>("barrel", tank_parts.barrel, player_entity, &barrel_q);
        validate_owned_part::<TankMuzzle>("muzzle", tank_parts.muzzle, player_entity, &muzzle_q);
    }
}

fn validate_owned_part<T: Component>(
    part_label: &str,
    part_entity: Entity,
    expected_owner: Entity,
    query: &Query<&OwnedBy, With<T>>,
) {
    let Ok(owned_by) = query.get(part_entity) else {
        warn!(
            "Invariant violation: Player {:?} references missing {} entity {:?} in TankParts",
            expected_owner, part_label, part_entity
        );
        return;
    };

    if owned_by.entity != expected_owner {
        warn!(
            "Invariant violation: {} {:?} owned by {:?}, expected {:?}",
            part_label, part_entity, owned_by.entity, expected_owner
        );
    }
}
