use std::time::Duration;

use bevy::prelude::*;

use crate::components::{
    BurningEffect, DamageEffect, Effect, EffectType, Experience, FreezingEffect, Health,
    StatusEffects,
};
use crate::enemy::EnemyDamageEvent;
use crate::events::ProjectileHitEvent;

pub fn handle_projectile_collision(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    mut enemy_damaged_events: EventWriter<EnemyDamageEvent>,
    mut enemy_query: Query<(&mut Health, &mut StatusEffects, &Transform, &Experience)>,
    projectile_query: Query<(
        &DamageEffect,
        Option<&BurningEffect>,
        Option<&FreezingEffect>,
    )>,
) {
    for (event, id) in collision_events.read_with_id() {
        if let Ok((mut health, mut status, transform, experience)) =
            enemy_query.get_mut(event.enemy)
        {
            if let Ok((damage, burning_effect, freezing_effect)) =
                projectile_query.get(event.projectile)
            {
                enemy_damaged_events.send(EnemyDamageEvent {
                    enemy_entity: event.enemy,
                    damage_source: Some(event.projectile),
                    damage: damage.base_damage,
                });

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
                commands.entity(event.projectile).try_despawn();
            }
        }
    }
}
