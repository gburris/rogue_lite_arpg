use std::time::Duration;

use bevy::prelude::*;

use crate::components::{
    BurningEffect, DamageEffect, Effect, EffectType, Experience, FreezingEffect, Health,
    StatusEffects,
};
use crate::events::{EnemyDefeatedEvent, ProjectileHitEvent};
use crate::helpers::handle_enemy_death::handle_enemy_death;
use crate::resources::ProcessedProjectiles;

pub fn handle_projectile_collision(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    mut enemy_defeated_events: EventWriter<EnemyDefeatedEvent>,
    mut enemy_query: Query<(&mut Health, &mut StatusEffects, &Transform, &Experience)>,
    projectile_query: Query<(
        &DamageEffect,
        Option<&BurningEffect>,
        Option<&FreezingEffect>,
    )>,
    mut processed: ResMut<ProcessedProjectiles>,
) {
    for (event, id) in collision_events.read_with_id() {
        if processed.set.contains(&id) {
            continue;
        }

        if let Ok((mut health, mut status, transform, experience)) =
            enemy_query.get_mut(event.enemy)
        {
            if let Ok((damage, burning_effect, freezing_effect)) =
                projectile_query.get(event.projectile)
            {
                processed.set.insert(id);

                // Apply base damage
                health.hp -= damage.base_damage;

                // Handle burning effect if present
                if let Some(burning) = burning_effect {
                    status.effects.push(Effect {
                        effect_type: EffectType::Burning,
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        damage_per_second: burning.damage_per_second,
                    });
                }

                // Handle freezing effect if present
                if let Some(_freezing) = freezing_effect {
                    status.effects.push(Effect {
                        effect_type: EffectType::Slowed,
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        damage_per_second: 0.0,
                    });
                }

                if health.hp <= 0.0 {
                    if let Some(entity_commands) = commands.get_entity(event.enemy) {
                        commands.entity(event.enemy).despawn();
                        enemy_defeated_events.send(EnemyDefeatedEvent {
                            enemy_entity: event.enemy,
                            enemy_position: transform.translation,
                            exp_value: experience.base_exp,
                        });
                    }
                }

                commands.entity(event.projectile).despawn();
            }
        }
    }
}
