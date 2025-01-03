use crate::{
    status_effects::StatusEffectAppliedEffect,
    status_effects::{StatusEffectType, StatusEffects},
};
use bevy::prelude::*;

use super::StatusEffect;
pub fn handle_status_effect_applied(
    mut status_effect_applied_event: EventReader<StatusEffectAppliedEffect>,
    mut query: Query<(Entity, &mut StatusEffects)>,
) {
    for event in status_effect_applied_event.read() {
        if let Ok((_entity, mut status_effects)) = query.get_mut(event.entity) {
            // Create the new effect based on type
            let new_effect = match event.effect {
                StatusEffectType::Burning => StatusEffect {
                    effect_type: StatusEffectType::Burning,
                    duration: Timer::from_seconds(5.0, TimerMode::Once),
                    damage_per_second: 10.0, // Adjust damage value as needed
                },
                StatusEffectType::Slowed => StatusEffect {
                    effect_type: StatusEffectType::Slowed,
                    duration: Timer::from_seconds(3.0, TimerMode::Once),
                    damage_per_second: 0.0,
                },
                StatusEffectType::Stunned => StatusEffect {
                    effect_type: StatusEffectType::Stunned,
                    duration: Timer::from_seconds(1.5, TimerMode::Once),
                    damage_per_second: 0.0,
                },
            };

            // Add the effect to the entity's status effects
            status_effects.effects.push(new_effect);
        }
    }
}
