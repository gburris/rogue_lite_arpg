use bevy::prelude::*;

use crate::{
    econ::components::{Currency, Wallet},
    items::Autoloot,
    player::Player,
};

pub fn update_autoloot_currency(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Currency), (With<Autoloot>, With<Currency>)>,
    mut player_query: Query<(&mut Wallet, &Transform), With<Player>>,
) {
    if let Ok((mut player_wallet, player_transform)) = player_query.get_single_mut() {
        for (currency_entity, currency_transform, currency) in query.iter() {
            let distance = player_transform
                .translation
                .distance(currency_transform.translation);
            if distance <= 10.0 {
                warn!("Autolooting the coin");
                player_wallet.add_currency(currency);
                commands.entity(currency_entity).despawn_recursive();
            }
        }
    }
}
