use crate::{
    components::{Health, Speed},
    helpers::labels::GameCollisionLayer,
};
use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, RigidBody, Sensor};
use bevy::prelude::*;

#[derive(Component)]
#[require(
    Health,
    Speed,
    Collider(|| Collider::rectangle(300.0, 300.0)),
    RigidBody(|| RigidBody::Static),
    CollisionLayers(|| CollisionLayers::new(
                       GameCollisionLayer::Npc, 
                       [GameCollisionLayer::Player]))
)]
pub struct NPC;

#[derive(Component)]
#[require(
    Health,
    CollidingEntities,
    Sensor,
    Speed,
    Collider(||  Collider::circle(500.0)),
    CollisionLayers(||  CollisionLayers::new(
                        GameCollisionLayer::Interaction,
                        [GameCollisionLayer::Player],))

)]
pub struct NPCInteractionRadius;