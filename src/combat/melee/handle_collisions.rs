use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{combat::damage::AttemptDamageEvent, enemy::Enemy, player::Player};

use super::components::{calculate_damage, ActiveMeleeAttack, MeleeWeapon};

pub fn handle_melee_collisions(
    mut commands: Commands,
    mut melee_query: Query<(
        Entity,
        &MeleeWeapon,
        &mut ActiveMeleeAttack,
        &CollidingEntities,
    )>,
    enemy_query: Query<&Enemy>,
    player: Single<Entity, With<Player>>,
) {
    let player_entity = player.into_inner();

    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities) in
        melee_query.iter_mut()
    {
        for &colliding_entity in colliding_entities.iter() {
            if (enemy_query.contains(colliding_entity) || colliding_entity == player_entity)
                && (!active_melee_attack
                    .entities_damaged
                    .contains(&colliding_entity))
            {
                let damage = calculate_damage(melee_weapon.damage);
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage,
                        damage_source: Some(weapon_entity),
                    },
                    colliding_entity,
                );
                active_melee_attack
                    .entities_damaged
                    .insert(colliding_entity);
            }
        }
    }
}
