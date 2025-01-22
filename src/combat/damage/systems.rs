use bevy::prelude::*;

use crate::combat::{
    attributes::health::Health,
    damage::{
        components::Invulnerable,
        events::{DamageEvent, DefeatedEvent},
    },
    status_effects::{components::EffectsList, events::ApplyEffect},
};

use super::components::HasIFrames;

pub fn on_damage_event(
    damage_trigger: Trigger<DamageEvent>,
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

        // Damage event decides whether the entity becomes invulernable afterwards
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

// System to handle invulnerability duration and flashing
pub fn handle_invulnerability(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invulnerable, &mut Sprite)>,
) {
    for (entity, mut invulnerable, mut sprite) in query.iter_mut() {
        // Update main invulnerability timer
        invulnerable.total_time.tick(time.delta());

        // Update flash timer
        invulnerable.flash_timer.tick(time.delta());

        // Handle flashing
        if invulnerable.flash_timer.just_finished() {
            invulnerable.is_transparent = !invulnerable.is_transparent;
            sprite.color.set_alpha(if invulnerable.is_transparent {
                0.1
            } else {
                1.0
            });
        }

        // Remove invulnerability when timer is finished
        if invulnerable.total_time.finished() {
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

pub fn on_remove_invulnerable(
    trigger: Trigger<OnRemove, Invulnerable>,
    mut query: Query<&mut Sprite>,
) {
    // Ensure sprite is fully visible when invulnerability is removed
    query
        .get_mut(trigger.entity())
        .unwrap()
        .color
        .set_alpha(1.0);
}
