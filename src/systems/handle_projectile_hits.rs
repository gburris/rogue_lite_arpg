use std::time::Duration;

use bevy::prelude::*;

use crate::components::{BurningEffect, DamageEffect, Enemy, Health};
use crate::events::ProjectileHitEvent;
use crate::resources::ProcessedProjectiles;

pub fn handle_projectile_hits(
    mut commands: Commands,
    mut events: EventReader<ProjectileHitEvent>,
    mut enemies: Query<(&mut Enemy, &mut Health)>,
    projectiles: Query<(&DamageEffect, &BurningEffect)>,
    mut processed: ResMut<ProcessedProjectiles>,
    asset_server: Res<AssetServer>,
) {
    for (event, id) in events.read_with_id() {
        if processed.set.contains(&id) {
            println!("Skipping event, it's already been processed {}", id);
            continue; // Skip already processed projectiles
        }
        //If the enemy is here, continue
        if let Ok((_enemy, mut health)) = enemies.get_mut(event.enemy) {
            //If the projectile is on the event, continue
            if let Ok(effects) = projectiles.get(event.projectile) {
                println!(
                    "PROCESS COLLISION EVENT: Adding eventId to my hashset {}",
                    id
                );
                processed.set.insert(id);
                commands.entity(event.enemy).insert((
                    //TODO: The queries in this file suck and require access via .1. and .0. leading to unreadble code
                    BurningEffect {
                        damage_per_second: effects.1.damage_per_second,
                        tick_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                    },
                    Sprite::from_image(asset_server.load("merman_on_fire.png")),
                ));
                println!("Handling ProjectileHitEvent event: Damage Effect of projectile is found");
                health.hp -= effects.0.base_damage;
                if health.hp <= 0.0 {
                    println!("Handling ProjectileHitEvent event: Enemy is dead");
                    commands.entity(event.enemy).despawn();
                }
                println!(
                    "Handling ProjectileHitEvent event: enemy is not dead {} ",
                    health.hp
                );
                commands.entity(event.projectile).despawn();
            }
        }
        //Despawn the projectile entity once we finish process it's collision event fully
    }
}
