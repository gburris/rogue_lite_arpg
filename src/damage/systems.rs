use bevy::prelude::*;

use crate::{
    damage::{
        components::{Health, InvulnerableFromDamage},
        events::{DamageEvent, DefeatedEvent},
    },
    status_effects::{components::EffectsList, events::ApplyEffect},
};

pub fn on_damage_event(
    damage_trigger: Trigger<DamageEvent>,
    mut commands: Commands,
    mut damaged_query: Query<(&mut Health, Option<&InvulnerableFromDamage>)>,
    source_query: Query<&EffectsList>,
) {
    if let Ok((mut health, invulnerable)) = damaged_query.get_mut(damage_trigger.entity()) {
        if invulnerable.is_some() {
            return;
        }

        health.take_damage(damage_trigger.damage);

        // Damage event decides whether the entity becomes invulernable afterwards
        if damage_trigger.makes_invulnerable {
            commands
                .entity(damage_trigger.entity())
                .insert(InvulnerableFromDamage::default());
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
    mut query: Query<(Entity, &mut InvulnerableFromDamage, &mut Sprite)>,
) {
    for (entity, mut invulnerable, mut sprite) in query.iter_mut() {
        // Update main invulnerability timer
        invulnerable.timer.tick(time.delta());

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
        if invulnerable.timer.finished() {
            commands.entity(entity).remove::<InvulnerableFromDamage>();
            sprite.color.set_alpha(1.0); // Ensure sprite is fully visible when done
        }
    }
}
