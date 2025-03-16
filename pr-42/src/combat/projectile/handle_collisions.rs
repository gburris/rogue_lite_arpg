use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::{
    attributes::Health, damage::events::AttemptDamageEvent, projectile::components::*,
    shield::components::ProjectileReflection,
};

pub fn handle_projectile_collisions(
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &CollidingEntities, Entity)>,
    health_query: Query<&Health>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, colliding_entities, projectile_entity) in projectile_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            // If the thing we collide with has health, lets try to damage it!
            if health_query.contains(colliding_entity) {
                let damage = calculate_damage(projectile.damage);
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage,
                        damage_source: Some(projectile_entity),
                    },
                    colliding_entity,
                );
            }
            if reflector_query.contains(colliding_entity) {
                continue;
            }
            commands.entity(projectile_entity).despawn_recursive();
            return;
        }
    }
}
