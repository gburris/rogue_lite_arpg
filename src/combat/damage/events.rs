use bevy::prelude::*;

#[derive(Event)]
pub struct AttemptDamageEvent {
    pub damage: f32,
    pub damage_source: Option<Entity>, //Not all damage has a "Source" entity, like environmental damage or DoT effects
}

/**
 * While AttemptDamageEvent is sent any time a damage source interacts with an entity,
 * this event represents when that damage attempt succeeds
 */
#[derive(Event)]
pub struct DamageDealtEvent {
    pub damage: f32,
}

#[derive(Event)]
pub struct DefeatedEvent {
    pub entity: Entity,
}
