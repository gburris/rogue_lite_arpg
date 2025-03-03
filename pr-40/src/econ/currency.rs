use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{configuration::GameCollisionLayer, items::inventory::Inventory, player::Player};

#[derive(Component)]
#[require(
    Collider(|| Collider::circle(10.0)),
    CollisionLayers(|| CollisionLayers::new(
        GameCollisionLayer::Interaction,
        [GameCollisionLayer::Player]
    )),
    Sensor,
    CollidingEntities,
)]
pub struct Currency {
    pub value: u32,
}

pub fn handle_currency_collisions(
    mut commands: Commands,
    currency_query: Query<(Entity, &Currency, &CollidingEntities)>,
    player: Single<(Entity, &mut Inventory), With<Player>>,
) {
    let (player_entity, mut player_inventory) = player.into_inner();

    for (currency_entity, currency, colliding_entities) in currency_query.iter() {
        if colliding_entities.contains(&player_entity) {
            player_inventory.add_coins(currency.value);
            commands.entity(currency_entity).despawn_recursive();
        }
    }
}
