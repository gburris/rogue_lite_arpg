use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{combat::damage::events::AttemptDamageEvent, enemy::Enemy};

use super::components::{ActiveMeleeAttack, MeleeWeapon};

pub fn handle_melee_collisions(
    mut commands: Commands,
    melee_query: Query<(&MeleeWeapon, &CollidingEntities), With<ActiveMeleeAttack>>,
    enemy_query: Query<&Enemy>,
) {
    for (melee_weapon, colliding_entities) in melee_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            if enemy_query.contains(colliding_entity) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: melee_weapon.damage,
                        damage_source: None,
                    },
                    colliding_entity,
                );
            }
            return;
        }
    }
}
