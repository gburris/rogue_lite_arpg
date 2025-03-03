use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{combat::status_effects::components::EffectsList, despawn::components::LiveDuration};

#[derive(Bundle, Clone)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub sprite: Sprite,
    pub effects_list: EffectsList,
}

#[derive(Component, Clone)]
#[require(
    LiveDuration(|| LiveDuration::new(1.0)),
    Sensor,
    RigidBody(default_rigid_body),
    Collider(default_collider),
    CollidingEntities,
)]
pub struct Projectile {
    pub damage: (f32, f32),
}

fn default_collider() -> Collider {
    Collider::rectangle(10.0, 10.0)
}

fn default_rigid_body() -> RigidBody {
    RigidBody::Dynamic
}

pub fn calculate_damage(damage_range: (f32, f32)) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(damage_range.0..damage_range.1)
}
