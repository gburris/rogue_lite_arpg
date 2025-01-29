use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::{components::CollisionDamage, events::AttemptDamageEvent},
    enemy::Enemy,
};

use super::components::{ActiveMeleeAttack, MeleeWeapon};

pub fn handle_melee_collisions(
    mut commands: Commands,
    melee_query: Query<(&ActiveMeleeAttack, &CollisionDamage, &CollidingEntities)>,
    enemy_query: Query<&Enemy>,
) {
    for (attack, damage, colliding_entities) in melee_query.iter() {
        //Try to hit everyone colliding with us. Note, this will probably trigger for everyone we collide with
        //So swinging and hitting two enemies at the same time would trigger four executions of the loop
        warn!("trying to do melee damage");
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

            // don't despawn melee attack
            return;
        }
    }
}
