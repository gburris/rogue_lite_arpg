use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{damage::components::CollisionDamage, status_effects::components::EffectsList},
    configuration::GameCollisionLayer,
    despawn::components::{DespawnOnCollision, LiveDuration},
};

#[derive(Bundle, Clone)]
pub struct ProjectileBundle {
    pub sprite: Sprite,
    pub damage: CollisionDamage,
    pub effects_list: EffectsList,
}

#[derive(Component, Clone)]
#[require(
    LiveDuration(|| LiveDuration::new(1.0)),
    DespawnOnCollision,
    Sensor,
    RigidBody(default_rigid_body),
    Collider(default_collider),
    CollisionLayers(default_collision_layers)
)]
pub struct Projectile;

fn default_collider() -> Collider {
    Collider::rectangle(10.0, 10.0)
}

fn default_rigid_body() -> RigidBody {
    RigidBody::Dynamic
}

fn default_collision_layers() -> CollisionLayers {
    // Currently projectiles can only collide with enemies
    CollisionLayers::new(
        GameCollisionLayer::InAir,
        [GameCollisionLayer::Enemy, GameCollisionLayer::HighObstacle],
    )
}
