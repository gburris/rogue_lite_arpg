use crate::components::Enemy;
use crate::components::Projectile;
use crate::events::ProjectileHitEvent;
use bevy::prelude::*;

pub fn check_projectile_collision(
    mut collision_events: EventWriter<ProjectileHitEvent>,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    println!("check_projectile_collision");
    for (projectile_entity, projectile_transform) in projectile_query.iter() {
        println!("check_projectile_collision2");
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            println!("check_projectile_collision3");
            if projectile_transform
                .translation
                .distance(enemy_transform.translation)
                < 25.0
            {
                println!("check_projectile_collision4");
                collision_events.send(ProjectileHitEvent {
                    projectile: projectile_entity,
                    enemy: enemy_entity,
                });
            }
        }
    }
}
