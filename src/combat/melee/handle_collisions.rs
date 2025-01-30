use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::{components::CollisionDamage, events::AttemptDamageEvent},
    enemy::Enemy,
};

use super::components::ActiveMeleeAttack;

pub fn handle_melee_collisions(
    mut commands: Commands,
    melee_query: Query<(&CollisionDamage, &CollidingEntities), With<ActiveMeleeAttack>>,
    enemy_query: Query<&Enemy>,
) {
    for (damage, colliding_entities) in melee_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            if enemy_query.contains(colliding_entity) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: damage.damage,
                        damage_source: None,
                    },
                    colliding_entity,
                );
            }
            return;
        }
    }
}
