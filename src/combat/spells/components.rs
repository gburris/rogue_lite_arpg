use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{combat::projectile::components::Projectile, helpers::labels::GameCollisionLayer};

#[derive(Component, Clone)]
#[require(
    Projectile,
    RigidBody(default_spell_rigid_body),
    Collider(default_spell_collider),
    CollisionLayers(default_spell_collision_layers)
)]
pub enum Spell {
    Fireball,
    Icebolt,
}

fn default_spell_collider() -> Collider {
    Collider::rectangle(10.0, 10.0)
}

fn default_spell_rigid_body() -> RigidBody {
    RigidBody::Dynamic
}

fn default_spell_collision_layers() -> CollisionLayers {
    // Currently projectiles can only collide with enemies
    CollisionLayers::new(
        GameCollisionLayer::Projectile,
        [GameCollisionLayer::Enemy, GameCollisionLayer::Wall],
    )
}
