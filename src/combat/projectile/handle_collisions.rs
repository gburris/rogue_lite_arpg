use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::{
    damage::{AttemptDamageEvent, Damage}, projectile::components::*, shield::components::ProjectileReflection, Health
};

pub fn handle_projectile_collisions(
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &CollidingEntities, Entity)>,
    health_query: Query<&Health>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, colliding_entities, projectile_entity) in projectile_query.iter() {
        // ignore further collisions after ANY collision with the projectile
        if let Some(&colliding_entity) = colliding_entities.iter().next() {
            // If the thing we collide with has health, lets try to damage it!
            if health_query.contains(colliding_entity) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: Damage::Range(projectile.damage),
                        damage_source: Some(projectile_entity),
                    },
                    colliding_entity,
                );
            }
            if reflector_query.contains(colliding_entity) {
                continue;
            }
            commands.entity(projectile_entity).despawn_recursive();
        }
    }
}
