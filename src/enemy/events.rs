use bevy::prelude::*;

#[derive(Event)]
pub struct EnemyDefeatedEvent {
    pub enemy_entity: Entity,
    pub enemy_position: Vec3, // Useful for spawning death effects
    pub exp_value: u32,
}
