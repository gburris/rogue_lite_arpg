use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{combat::damage::events::AttemptDamageEvent, enemy::Enemy, player::Player};

use super::components::{ActiveMeleeAttack, MeleeWeapon};

pub fn handle_melee_collisions(
    mut commands: Commands,
    melee_query: Query<(Entity, &MeleeWeapon, &CollidingEntities), With<ActiveMeleeAttack>>,
    enemy_query: Query<&Enemy>,
    player: Single<Entity, With<Player>>,
) {
    let player_entity = player.into_inner();

    for (weapon_entity, melee_weapon, colliding_entities) in melee_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            if enemy_query.contains(colliding_entity) || colliding_entity == player_entity {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: melee_weapon.damage,
                        damage_source: Some(weapon_entity),
                    },
                    colliding_entity,
                );
            }
        }
    }
}
