use avian2d::prelude::*;
use bevy::prelude::*;

use crate::helpers::labels::GameCollisionLayer;

/**
 * Portals represent any "warping device" in the game, currently spawning a new zone when entered
 */
#[derive(Component)]
#[require(
    RigidBody(default_rigid_body),
    Collider(default_collider),
    CollisionLayers(default_collision_layers),
    Sensor
)]
pub enum Portal {
    StartingPortal,
    WarpZone,
}

fn default_collider() -> Collider {
    Collider::rectangle(100.0, 100.0)
}

fn default_rigid_body() -> RigidBody {
    RigidBody::Static
}

fn default_collision_layers() -> CollisionLayers {
    // Portals only collide with the player, and are sensors since we don't actually want collisions
    CollisionLayers::new(GameCollisionLayer::Portal, [GameCollisionLayer::Player])
}
