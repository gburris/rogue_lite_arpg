use bevy::prelude::*;

use crate::components::{BurningEffect, DamageEffect, FreezingEffect};
use crate::enemy::EnemyDamageEvent;
use crate::events::ProjectileHitEvent;
use crate::status_effects::StatusEffectAppliedEvent;
use crate::status_effects::StatusEffectType;

pub fn handle_projectile_collision(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    mut enemy_damaged_events: EventWriter<EnemyDamageEvent>,
    mut status_applied_events: EventWriter<StatusEffectAppliedEvent>,
    projectile_query: Query<(
        &DamageEffect,
        Option<&BurningEffect>,
        Option<&FreezingEffect>,
    )>,
) {
    for event in collision_events.read() {
        if let Ok((damage, burning_effect, freezing_effect)) =
            projectile_query.get(event.projectile)
        {
            enemy_damaged_events.send(EnemyDamageEvent {
                enemy_entity: event.enemy,
                damage_source: Some(event.projectile),
                damage: damage.base_damage,
            });

            // Handle burning effect if present
            if let Some(_burning) = burning_effect {
                status_applied_events.send(StatusEffectAppliedEvent {
                    entity: event.enemy,
                    effect: StatusEffectType::Burning,
                });
            }

            // Handle freezing effect if present
            if let Some(_freezing) = freezing_effect {
                status_applied_events.send(StatusEffectAppliedEvent {
                    entity: event.enemy,
                    effect: StatusEffectType::Slowed,
                });
            }
            commands.entity(event.projectile).try_despawn();
        }
    }
}
