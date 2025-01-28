use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, RigidBody, Sensor};
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

#[derive(Component)]
#[require(
    Collider(|| Collider::rectangle(180.0, 50.0)),
    RigidBody(|| RigidBody::Static),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::LowObstacle, GameCollisionLayer::LOW_OBSTACLE_FILTERS))
)]
pub struct Chest;

#[derive(Component)]
#[require(
    CollidingEntities,
    Sensor,
    Collider(||  Collider::circle(100.0)),
    CollisionLayers(||  CollisionLayers::new(
                        GameCollisionLayer::Interaction,
                        [GameCollisionLayer::Player],))

)]
pub struct ChestInteractionRadius;

#[derive(Event)]
pub struct OpenChest {
    pub chest_entity: Entity,
}
