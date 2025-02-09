use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    econ::components::{Currency, Wallet},
    items::Autoloot,
    player::Player,
};

pub fn update_autoloot_currency(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Currency, &Collider), (With<Autoloot>, With<Currency>)>,
    mut player_query: Query<(&mut Wallet, &Collider, &Transform), With<Player>>,
) {
    if let Ok((mut player_wallet, player_collider, player_transform)) =
        player_query.get_single_mut()
    {
        for (currency_entity, currency_transform, currency, _currency_collider) in query.iter() {
            // Check if the currency's position is inside the player's collider
            if player_collider.contains_point(
                player_transform.translation.truncate(),
                player_transform.rotation,
                currency_transform.translation.truncate(),
            ) {
                player_wallet.add_currency(currency);
                commands.entity(currency_entity).despawn_recursive();
            }
        }
    }
}
