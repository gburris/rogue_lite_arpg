use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::attributes::Health, configuration::GameCollisionLayer,
    movement::components::SimpleMotion,
};

#[derive(Component)]
#[require(
    Health,
    SimpleMotion,
    Collider(|| Collider::rectangle(300.0, 300.0)),
    RigidBody(|| RigidBody::Kinematic),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Grounded, [GameCollisionLayer::Grounded, GameCollisionLayer::InAir]))
)]
pub struct NPC;

#[derive(Component)]
#[require(
    CollidingEntities,
    Sensor,
    Collider(||  Collider::circle(500.0)),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, [GameCollisionLayer::Player]))

)]
pub struct NPCInteractionRadius;
