use std::time::Duration;

use bevy::prelude::*;

use crate::components::{BurningEffect, Fireball};
use crate::events::ProjectileHitEvent;

pub fn apply_projectile_effects(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    projectile_query: Query<(Entity, Option<&Fireball>, Option<&BurningEffect>)>,
) {
    for event in collision_events.read() {
        if let Ok((_, fireball, burning_effect)) = projectile_query.get(event.projectile) {
            // Apply fireball effects
            //TODO: THIS IS REALLY BAD, IT JUST MAKES A DEEP COPY OF THE BURNING EFFECT.
            if fireball.is_some() {
                if let Some(burning_effect) = burning_effect {
                    commands.entity(event.enemy).insert(BurningEffect {
                        damage_per_second: burning_effect.damage_per_second,
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                    });
                }
            }
        }
    }
}


