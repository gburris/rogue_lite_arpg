use bevy::prelude::*;

#[derive(Event)]
pub struct DamageEvent {
    pub damage: f32,
    pub damage_source: Option<Entity>, //Not all damage has a "Source" entity, like environmental damage or DoT effects
}

#[derive(Event)]
pub struct DefeatedEvent;
