use bevy::prelude::*;

use crate::components::{BurningEffect, Enemy, Health};

pub fn process_burning(
    mut commands: Commands,
    time: Res<Time>,
    mut burning_effect_query: Query<(Entity, &mut BurningEffect, &mut Enemy, &mut Health)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, mut burning, mut enemy, mut health) in &mut burning_effect_query {
        // Tick the burning effect's timer
        burning.duration.tick(time.delta());
        burning.tick_timer.tick(time.delta());

        // Apply damage if the tick_timer just finished (every second)
        if burning.tick_timer.just_finished() {
            health.hp -= burning.damage_per_second;

            println!(
                "Burning Effect ticking on the enemy, dealing {:.2} damage to monster with health {:.2}",
                burning.damage_per_second, health.hp
            );

            // Check if the enemy is dead
            if health.hp <= 0.0 {
                println!("Burning Effect killed the enemy");
                commands.entity(entity).despawn();
                continue; // Skip further processing for this entity
            }
        }

        // Check if the burning effect duration has ended
        if burning.duration.finished() {
            println!("Burning Effect expired on the enemy");
            commands
                .entity(entity)
                .remove::<BurningEffect>()
                .insert(Sprite::from_image(asset_server.load("merman.png")));
        }
    }
}
