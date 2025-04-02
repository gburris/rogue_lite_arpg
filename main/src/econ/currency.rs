use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::{GameCollisionLayer, YSort},
    items::inventory::Inventory,
    player::{Player, PlayerCollider},
};

#[derive(Component)]
#[require(
    RigidBody,
    Collider(|| Collider::circle(10.0)),
    CollisionLayers(|| CollisionLayers::new(
        [GameCollisionLayer::Grounded],
        [GameCollisionLayer::PlayerCollider, GameCollisionLayer::HighObstacle, GameCollisionLayer::LowObstacle]
    )),
    CollidingEntities,
    LockedAxes(|| LockedAxes::new().lock_rotation()),
    LinearDamping(|| LinearDamping(2.0)),
    TranslationExtrapolation,
    // Don't let currency move the player upon collision
    Dominance(|| Dominance(-1)),
    YSort,
)]
pub struct Currency {
    pub value: u32,
}

pub fn handle_currency_collisions(
    mut commands: Commands,
    currency_query: Query<(Entity, &Currency, &CollidingEntities)>,
    mut player_inventory: Single<&mut Inventory, With<Player>>,
    player_collider: Query<Entity, With<PlayerCollider>>,
) {
    let Ok(player_collider_entity) = player_collider.get_single() else {
        return;
    };

    for (currency_entity, currency, colliding_entities) in currency_query.iter() {
        if colliding_entities.contains(&player_collider_entity) {
            player_inventory.add_coins(currency.value);
            commands.entity(currency_entity).despawn_recursive();
        }
    }
}
