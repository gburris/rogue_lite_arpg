use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{
        damage::{components::CollisionDamage, events::AttemptDamageEvent},
        projectile::components::*,
    },
    enemy::Enemy,
    player::Player,
};

pub fn handle_projectile_collisions(
    mut commands: Commands,
    projectile_query: Query<(&CollisionDamage, &CollidingEntities, Entity), With<Projectile>>,
    enemy_query: Query<&Enemy>,
    player: Single<Entity, With<Player>>,
) {
    let player_entity = player.into_inner();
    for (collision_damage, colliding_entities, projectile_entity) in projectile_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            if enemy_query.contains(colliding_entity) || colliding_entity == player_entity {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: collision_damage.damage,
                        damage_source: Some(projectile_entity),
                    },
                    colliding_entity,
                );
            }
            // despawn projectile and ignore further collisions after ANY collision
            commands.entity(projectile_entity).despawn_recursive();
            return;
        }
    }
}
