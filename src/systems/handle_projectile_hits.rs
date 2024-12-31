use std::time::Duration;

use bevy::prelude::*;

use crate::components::{BurningEffect, DamageEffect, Enemy};
use crate::events::ProjectileHitEvent;
use crate::resources::ProcessedProjectiles;

pub fn handle_projectile_hits(
    mut commands: Commands,
    mut events: EventReader<ProjectileHitEvent>,
    mut enemies: Query<&mut Enemy>,
    projectiles: Query<(&DamageEffect, &BurningEffect)>,
    mut processed: ResMut<ProcessedProjectiles>,
) {
    for (event, id) in events.read_with_id() {
        if processed.set.contains(&id) {
            println!("Skipping event, it's already been processed {}", id);
            continue; // Skip already processed projectiles
        }
        //If the enemy is here, continue
        if let Ok(mut enemy) = enemies.get_mut(event.enemy) {
            //If the projectile is on the event, continue
            if let Ok(effects) = projectiles.get(event.projectile) {
                println!(
                    "PROCESS COLLISION EVENT: Adding eventId to my hashset {}",
                    id
                );
                processed.set.insert(id);
                commands.entity(event.enemy).insert(BurningEffect {
                    damage_per_second: effects.1.damage_per_second,
                    tick_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                    duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                });
                println!("Handling ProjectileHitEvent event: Damage Effect of projectile is found");
                enemy.health -= effects.0.base_damage;
                if enemy.health <= 0.0 {
                    println!("Handling ProjectileHitEvent event: Enemy is dead");
                    commands.entity(event.enemy).despawn();
                }
                println!(
                    "Handling ProjectileHitEvent event: enemy is not dead {} ",
                    enemy.health
                );
                commands.entity(event.projectile).despawn();
            }
        }
        //Despawn the projectile entity once we finish process it's collision event fully
    }
}
