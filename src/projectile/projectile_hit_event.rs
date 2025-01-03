use bevy::prelude::*;

#[derive(Event)]
pub struct ProjectileHitEvent {
    pub projectile: Entity,
    pub enemy: Entity,
}