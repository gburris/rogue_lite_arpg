use bevy::prelude::*;

use crate::despawn::components::LiveDuration;

use super::{
    components::{AppliedStatus, StatusEffect},
    events::ApplyEffects,
};

/**
 * Distributes the list of statuses as status events
 */
pub fn on_effect_apply(
    trigger: Trigger<ApplyEffects>,
    mut commands: Commands,
    status_source_query: Query<&Children>,
    effects_query: Query<(Entity, &StatusEffect)>,
) {
    // Find all children of entity that is applying the status effects
    if let Ok(children) = status_source_query.get(trigger.effect_source) {
        for &child in children.iter() {
            // If the child is a status effect
            if let Ok((effect_entity, effect)) = effects_query.get(child) {
                // Move status effect to entity where it will be applied
                commands
                    .entity(trigger.entity())
                    .add_child(effect_entity.clone());

                // Mark it as an applied status and give it a time to live
                commands
                    .entity(effect_entity)
                    .insert((AppliedStatus, LiveDuration::new(effect.duration)));
            }
        }
    }
}
