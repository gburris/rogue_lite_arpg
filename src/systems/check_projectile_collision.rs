use crate::components::{Enemy, Projectile};
use crate::events::ProjectileHitEvent;
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn check_projectile_collision(
    mut collision_events_started: EventReader<CollisionStarted>,
    mut projectile_hit_event: EventWriter<ProjectileHitEvent>,
    projectile_query: Query<Entity, With<Projectile>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        // Perform collision from e1 -> e2 and e2 -> e1 so both have the others damage applied
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            // Checks if one of the entities is a projectile and one is an enemy
            if let Ok(projectile_entity) = projectile_query.get(e1) {
                if let Ok(enemy_entity) = enemy_query.get(e2) {
                    projectile_hit_event.send(ProjectileHitEvent {
                        projectile: projectile_entity,
                        enemy: enemy_entity,
                    });
                }
            }
        }
    }
}
