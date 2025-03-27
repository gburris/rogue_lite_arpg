use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::{GameCollisionLayer, YSort},
    items::inventory::Inventory,
    player::{Player, PlayerCollider},
};

#[derive(Component)]
#[require(
    RigidBody(|| RigidBody::Dynamic),
    Collider(|| Collider::circle(10.0)),
    CollisionLayers(|| CollisionLayers::new(
        [GameCollisionLayer::Grounded],
        [GameCollisionLayer::PlayerCollider, GameCollisionLayer::HighObstacle, GameCollisionLayer::LowObstacle]
    )),
    CollidingEntities,
    LinearDamping(|| LinearDamping(2.0)),
    TranslationInterpolation,
    YSort
)]
pub struct Currency {
    pub value: u32,
}

pub fn handle_currency_collisions(
    mut commands: Commands,
    currency_query: Query<(Entity, &Currency, &CollidingEntities)>,
    mut player_inventory: Single<&mut Inventory, With<Player>>,
    player_collider: Single<Entity, With<PlayerCollider>>,
) {
    let player_collider_entity = player_collider.into_inner();
    for (currency_entity, currency, colliding_entities) in currency_query.iter() {
        if colliding_entities.contains(&player_collider_entity) {
            player_inventory.add_coins(currency.value);
            commands.entity(currency_entity).despawn_recursive();
        }
    }
}
