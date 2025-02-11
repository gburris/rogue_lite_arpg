use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    econ::components::{Currency, Wallet},
    items::Autoloot,
    player::Player,
};

pub fn update_autoloot_currency(
    mut commands: Commands,
    currency_query: Query<
        (Entity, &Currency, &CollidingEntities),
        (With<Autoloot>, With<Currency>),
    >,
    mut player_query: Query<(&mut Wallet, &Transform), With<Player>>,
) {
    if let Ok((mut player_wallet, player_collider, player_transform)) =
        player_query.get_single_mut()
    {
        for (currency_entity, currency_transform, colliding_entities) in currency_query.iter() {
            if colliding_entities.contains(&player_entity) {
                player_wallet.add_currency(currency);
                commands.entity(currency_entity).despawn_recursive();
            }
        }
    }
}
