use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::{components::CollisionDamage, events::AttemptDamageEvent},
    despawn::components::DespawnOnCollision,
    map::{components::Portal, events::CreateInstanceEvent},
    player::Player,
};

/**
 * Main collision loop in game, dispatches various collisions to other systems via events
 */
pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events_started: EventReader<CollisionStarted>,
    damager_query: Query<(Entity, &CollisionDamage)>,
    portal_query: Query<&Portal>,
    player_query: Query<Entity, With<Player>>,
    despawn_on_collision_query: Query<&DespawnOnCollision>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        // Perform collision from e1 -> e2 and e2 -> e1 so both have the others damage applied
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            //
            if let Ok((damager_entity, collision_damage)) = damager_query.get(e1) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: collision_damage.damage,
                        damage_source: Some(damager_entity),
                    },
                    e2,
                );
            }

            if let Ok(_) = despawn_on_collision_query.get(e1) {
                commands.entity(e1).despawn_recursive();
            }

            if let Ok(_portal) = portal_query.get(e1) {
                if let Ok(_player_entity) = player_query.get(e2) {
                    commands.trigger(CreateInstanceEvent);
                    break;
                }
            }
        }
    }
}
