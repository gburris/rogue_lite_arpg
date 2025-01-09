use bevy::prelude::*;

#[derive(Event)]
pub struct DamageEvent {
    pub entity: Entity,
    pub damage_source: Option<Entity>, //Not all damage has a "Source" entity, like environmental damage or DoT effects
    pub damage: f32,
}

#[derive(Event)]
pub struct EnemyDefeatedEvent {
    pub enemy_entity: Entity,
    pub enemy_position: Vec3, // Useful for spawning death effects
    pub exp_value: u32,
}

#[derive(Event)]
pub struct DespawnAllEnemies;
