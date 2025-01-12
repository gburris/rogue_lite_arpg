use bevy::prelude::*;

use crate::{
    damage::{
        components::Health,
        events::{DamageEvent, DefeatedEvent},
    },
    status_effects::{components::EffectsList, events::ApplyEffect},
};

pub fn handle_damage(
    trigger: Trigger<DamageEvent>,
    mut commands: Commands,
    mut damaged_query: Query<&mut Health>,
    source_query: Query<&EffectsList>,
) {
    if let Ok(mut health) = damaged_query.get_mut(trigger.entity()) {
        health.take_damage(trigger.damage);

        if health.hp == 0.0 {
            commands.trigger_targets(DefeatedEvent, trigger.entity());
        } else if let Some(source_entity) = trigger.damage_source {
            // If entity is still alive and damage source exists and has effects list, we apply status effects
            if let Ok(effects_list) = source_query.get(source_entity) {
                commands.trigger_targets(
                    ApplyEffect {
                        effect: effects_list.effects.clone(),
                    },
                    trigger.entity(),
                );
            }
        }
    }
}
