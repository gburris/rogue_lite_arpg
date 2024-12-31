use crate::components::{Collider, Enemy, Projectile};
use crate::events::ProjectileHitEvent;
use bevy::prelude::*;

pub fn check_projectile_collision(
    mut collision_events: EventWriter<ProjectileHitEvent>,
    projectile_query: Query<(Entity, &Transform, &Collider), With<Projectile>>,
    enemy_query: Query<(Entity, &Transform, &Collider), With<Enemy>>,
) {
    for (projectile_entity, projectile_transform, projectile_collider) in projectile_query.iter() {
        for (enemy_entity, enemy_transform, enemy_collider) in enemy_query.iter() {
            let collision = check_aabb_collision(
                projectile_transform.translation,
                projectile_collider.size,
                enemy_transform.translation,
                enemy_collider.size,
            );
            if collision {
                println!("SENDING COLLISION EVENT");
                collision_events.send(ProjectileHitEvent {
                    projectile: projectile_entity,
                    enemy: enemy_entity,
                });
            }
        }
    }
}

pub fn check_aabb_collision(pos_a: Vec3, size_a: Vec2, pos_b: Vec3, size_b: Vec2) -> bool {
    // Calculate the half-sizes
    let half_size_a = size_a / 2.0;
    let half_size_b = size_b / 2.0;

    // Calculate the bounds for both boxes
    let min_a = Vec2::new(pos_a.x - half_size_a.x, pos_a.y - half_size_a.y);
    let max_a = Vec2::new(pos_a.x + half_size_a.x, pos_a.y + half_size_a.y);
    let min_b = Vec2::new(pos_b.x - half_size_b.x, pos_b.y - half_size_b.y);
    let max_b = Vec2::new(pos_b.x + half_size_b.x, pos_b.y + half_size_b.y);

    // Check for overlap on both axes
    // Boxes overlap if max of one is greater than min of other on both axes
    min_a.x <= max_b.x && max_a.x >= min_b.x && min_a.y <= max_b.y && max_a.y >= min_b.y
}
