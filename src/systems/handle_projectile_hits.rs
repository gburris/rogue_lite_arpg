use std::time::Duration;

use bevy::prelude::*;

use crate::components::{
    BurningEffect, DamageEffect, Effect, EffectType, FreezingEffect, Health, StatusEffects,
};
use crate::events::ProjectileHitEvent;
use crate::resources::ProcessedProjectiles;

pub fn handle_projectile_collision(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    mut enemy_query: Query<(&mut Health, &mut StatusEffects)>,
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

        if let Ok((mut health, mut status)) = enemy_query.get_mut(event.enemy) {
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
                    commands.entity(event.enemy).despawn();
                }

                commands.entity(event.projectile).despawn();
            }
        }
    }
}
