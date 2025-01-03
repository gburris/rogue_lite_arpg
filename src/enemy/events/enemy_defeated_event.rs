use bevy::prelude::*;
use bevy::{math::Vec3, prelude::Event};

#[derive(Event)]
pub struct EnemyDefeatedEvent {
    pub enemy_entity: Entity,
    pub enemy_position: Vec3, // Useful for spawning death effects
    pub exp_value: u32,
}
