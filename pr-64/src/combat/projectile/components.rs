use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{combat::status_effects::components::EffectsList, despawn::components::LiveDuration};

#[derive(Bundle, Clone)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub sprite: Sprite,
    pub effects_list: EffectsList,
}

#[derive(Component, Clone)]
#[require(
    LiveDuration::new(1.0),
    Sensor,
    RigidBody,
    Collider::rectangle(10.0, 10.0),
    CollidingEntities
)]
pub struct Projectile {
    pub damage: (f32, f32),
}
