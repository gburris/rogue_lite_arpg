use bevy::prelude::*;

#[derive(Event)]
pub struct DamageEvent {
    pub damage: f32,
    pub damage_source: Option<Entity>, //Not all damage has a "Source" entity, like environmental damage or DoT effects
}

/**
 * While DamageEvent is sent to the entitiy taken damage,
 * this event is used for the entity dealing the damage for any cleanup.
 * Add observers to the entity if they have specific logic in this situation
 * Ex. Projectiles instantly despawn once they deal damage rather than using the concept of "Health"
 */
#[derive(Event)]
pub struct DealtDamageEvent {
    pub damage: f32,
    pub damaged_entity: Entity,
}

#[derive(Event)]
pub struct DefeatedEvent;
