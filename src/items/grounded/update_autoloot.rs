use avian2d::prelude::CollidingEntities;
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
    mut player_query: Query<(Entity, &mut Wallet), With<Player>>,
) {
    if let Ok((player_entity, mut player_wallet)) = player_query.get_single_mut() {
        for (currency_entity, currency, colliding_entities) in currency_query.iter() {
            if colliding_entities.contains(&player_entity) {
                player_wallet.add_currency(currency);
                commands.entity(currency_entity).despawn_recursive();
            }
        }
    }
}
