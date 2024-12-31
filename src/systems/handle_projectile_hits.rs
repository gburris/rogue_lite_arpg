use bevy::prelude::*;

use crate::components::{DamageEffect, Enemy};
use crate::events::ProjectileHitEvent;

pub fn handle_projectile_hits(
    mut commands: Commands,
    mut events: EventReader<ProjectileHitEvent>,
    mut enemies: Query<&mut Enemy>,
    projectiles: Query<&DamageEffect>,
) {
    for event in events.read() {
        if let Ok(mut enemy) = enemies.get_mut(event.enemy) {
            if let Ok(damage_effect) = projectiles.get(event.projectile) {
                enemy.health -= damage_effect.base_damage;
                if enemy.health <= 0.0 {
                    commands.entity(event.enemy).despawn();
                }
            }
        }
        commands.entity(event.projectile).despawn();
    }
}
