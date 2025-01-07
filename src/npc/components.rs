use crate::{
    components::{Health, Speed},
    helpers::labels::GameCollisionLayer,
};
use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, Sensor};
use bevy::prelude::*;

#[derive(Component)]
#[require(Health, Speed, Collider, CollisionLayers(default_collision_layers))]
pub struct NPC;

#[derive(Component)]
#[require(
    Health,
    Sensor,
    Speed,
    Collider(default_radius_collider),
    CollisionLayers(interaction_collision_layers)
)]
pub struct NPCInteractionRadius;

fn default_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameCollisionLayer::Npc, [GameCollisionLayer::Player])
}

fn interaction_collision_layers() -> CollisionLayers {
    CollisionLayers::new(
        GameCollisionLayer::Interaction,
        [GameCollisionLayer::Player],
    )
}

fn default_radius_collider() -> Collider {
    // Portals only collide with the player, and are sensors since we don't actually want collisions
    Collider::circle(500.0)
}
