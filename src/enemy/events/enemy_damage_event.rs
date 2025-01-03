use bevy::prelude::*;
use bevy::prelude::Event;

#[derive(Event)]
pub struct EnemyDamageEvent {
    pub enemy_entity: Entity,
    pub damage_source: Option<Entity>, //Not all damage has a "Source" entity, like environmental damage or DoT effects
    pub damage: f32,
}
