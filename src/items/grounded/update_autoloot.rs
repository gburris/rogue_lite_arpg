use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

use crate::{
    econ::components::Currency,
    items::{inventory::Inventory, Autoloot},
    player::Player,
};

pub fn update_autoloot_currency(
    mut commands: Commands,
    currency_query: Query<
        (Entity, &Currency, &CollidingEntities),
        (With<Autoloot>, With<Currency>),
    >,
    mut player_query: Query<(Entity, &mut Inventory), With<Player>>,
) {
    if let Ok((player_entity, mut player_inventory)) = player_query.get_single_mut() {
        for (currency_entity, currency, colliding_entities) in currency_query.iter() {
            if colliding_entities.contains(&player_entity) {
                player_inventory.add_coins(currency.value);
                commands.entity(currency_entity).despawn_recursive();
            }
        }
    }
}
