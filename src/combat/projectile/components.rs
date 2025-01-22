use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{damage::components::CollisionDamage, status_effects::components::EffectsList},
    despawn::components::LiveDuration,
    helpers::labels::GameCollisionLayer,
};

#[derive(Bundle, Clone)]
pub struct ProjectileBundle {
    pub sprite: Sprite,
    pub damage: CollisionDamage,
    pub effects_list: EffectsList,
}

#[derive(Component, Clone)]
#[require(
    LiveDuration,
    Sensor,
    RigidBody(default_spell_rigid_body),
    Collider(default_spell_collider),
    CollisionLayers(default_spell_collision_layers)
)]
pub struct Projectile;

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
