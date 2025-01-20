use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::components::Health, helpers::labels::GameCollisionLayer,
    movement::components::SimpleMotion,
};

#[derive(Component)]
#[require(
    Health,
    SimpleMotion,
    Collider(|| Collider::rectangle(300.0, 300.0)),
    RigidBody(|| RigidBody::Kinematic),
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
    Collider(||  Collider::circle(500.0)),
    CollisionLayers(||  CollisionLayers::new(
                        GameCollisionLayer::Interaction,
                        [GameCollisionLayer::Player],))

)]
pub struct NPCInteractionRadius;
