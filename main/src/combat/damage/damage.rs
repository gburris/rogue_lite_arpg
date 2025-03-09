use avian2d::prelude::*;

use bevy::prelude::*;

use crate::{
    combat::{
        attributes::health::Health,
        damage::{HasIFrames, Invulnerable},
        status_effects::{components::EffectsList, events::ApplyEffect},
    },
    configuration::GameCollisionLayer,
};

#[derive(PartialEq)]
pub enum DamageSource {
    Player,
    Enemy,
    NPC,
    Environment,
}

impl From<DamageSource> for LayerMask {
    fn from(source: DamageSource) -> Self {
        match source {
            DamageSource::Player => GameCollisionLayer::Enemy.to_bits(),
            DamageSource::Enemy => GameCollisionLayer::Player.to_bits(),
            DamageSource::NPC => GameCollisionLayer::Enemy.to_bits(),
            DamageSource::Environment => {
                // Combine both Player and Enemy layers for Environment
                GameCollisionLayer::Enemy.to_bits() | GameCollisionLayer::Player.to_bits()
            }
        }
        .into()
    }
}

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
pub struct DefeatedEvent;

pub fn on_damage_event(
    damage_trigger: Trigger<AttemptDamageEvent>,
    mut commands: Commands,
    mut damaged_query: Query<(&mut Health, Option<&HasIFrames>, Option<&Invulnerable>)>,
    source_query: Query<&EffectsList>,
) {
    if let Ok((mut health, has_iframes, invulnerable)) =
        damaged_query.get_mut(damage_trigger.entity())
    {
        if invulnerable.is_some() {
            return;
        }

        health.take_damage(damage_trigger.damage);

        // Because AttemptDamageEvent may not result in damage being applied (invulnerable or entity without health)
        // we send this event for guranteed "X damage has been done". Proper change detection added to bevy would mean this isn't needed
        commands.trigger_targets(
            DamageDealtEvent {
                damage: damage_trigger.damage,
            },
            damage_trigger.entity(),
        );

        // Entities have to "opt-in" to having iframes. Right now that is only the player
        if let Some(iframes) = has_iframes {
            commands
                .entity(damage_trigger.entity())
                .insert(Invulnerable::new(iframes));
        }

        if health.hp == 0.0 {
            commands.trigger_targets(DefeatedEvent, damage_trigger.entity());
        } else if let Some(source_entity) = damage_trigger.damage_source {
            // If entity is still alive and damage source exists and has effects list, we apply status effects
            if let Ok(effects_list) = source_query.get(source_entity) {
                commands.trigger_targets(
                    ApplyEffect {
                        effect: effects_list.effects.clone(),
                    },
                    damage_trigger.entity(),
                );
            }
        }
    }
}
