use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::{
        components::CollisionDamage,
        events::{DamageEvent, DealtDamageEvent},
    },
    map::{
        components::Portal,
        events::{StartRunEvent, WarpZoneEnterEvent},
    },
    player::Player,
};

/**
 * Main collision loop in game, dispatches various collisions to other systems via events
 */
pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events_started: EventReader<CollisionStarted>,
    mut warpzone_enter_event_writer: EventWriter<WarpZoneEnterEvent>,
    mut run_start_portal_event_writer: EventWriter<StartRunEvent>,
    damager_query: Query<(Entity, &CollisionDamage)>,
    portal_query: Query<&Portal>,
    player_query: Query<Entity, With<Player>>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        // Perform collision from e1 -> e2 and e2 -> e1 so both have the others damage applied
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            //
            if let Ok((damager_entity, collision_damage)) = damager_query.get(e1) {
                commands.trigger_targets(
                    DamageEvent {
                        damage: collision_damage.damage,
                        damage_source: Some(damager_entity),
                    },
                    e2,
                );

                commands.trigger_targets(DealtDamageEvent, damager_entity);

                // Even if e1 -> e2 does damage, we need to check if e2 -> e1 does damage too
                continue;
            }

            if let Ok(portal) = portal_query.get(e1) {
                if let Ok(_player_entity) = player_query.get(e2) {
                    match portal {
                        Portal::StartingPortal => {
                            debug!("Found collision with starting portal");
                            run_start_portal_event_writer.send(StartRunEvent);
                        }
                        Portal::WarpZone => {
                            debug!("Found collision with warpzone");
                            warpzone_enter_event_writer.send(WarpZoneEnterEvent);
                        }
                    }
                    // Once we find a match we go to the next collision
                    break;
                }
            }
        }
    }
}
