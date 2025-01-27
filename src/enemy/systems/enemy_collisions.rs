use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::{components::CollisionDamage, events::AttemptDamageEvent},
    enemy::Enemy,
    player::Player,
};

pub fn handle_enemy_collisions(
    mut commands: Commands,
    enemy_query: Query<(&CollisionDamage, &CollidingEntities, Entity), With<Enemy>>,
    player: Single<Entity, With<Player>>,
) {
    let player_entity = player.into_inner();

    for (collision_damage, colliding_entities, enemy_entity) in enemy_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            if colliding_entity == player_entity {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: collision_damage.damage,
                        damage_source: Some(enemy_entity),
                    },
                    player_entity,
                );
            }
        }
    }
}
