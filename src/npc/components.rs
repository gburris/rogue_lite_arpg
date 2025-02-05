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
    Collider(|| Collider::rectangle(32.0, 32.0)),
    RigidBody(|| RigidBody::Kinematic),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Grounded, [GameCollisionLayer::Grounded, GameCollisionLayer::InAir]))
)]
pub struct NPC;

#[derive(Component)]
#[require(
    CollidingEntities,
    Sensor,
    Collider(||  Collider::circle(70.0)),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, [GameCollisionLayer::Player]))

)]
pub struct NPCInteractionRadius;
