use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    map::{
        components::Portal,
        events::{StartRunEvent, WarpZoneEnterEvent},
    },
    npc::NPC,
    player::Player,
    projectile::{events::ProjectileHitEvent, Projectile},
};

/**
 * Main collision loop in game, dispatches various collisions to other systems via events
 */
pub fn handle_collisions(
    mut collision_events_started: EventReader<CollisionStarted>,
    mut projectile_hit_event: EventWriter<ProjectileHitEvent>,
    mut warpzone_enter_event_writer: EventWriter<WarpZoneEnterEvent>,
    mut run_start_portal_event_writer: EventWriter<StartRunEvent>,
    projectile_query: Query<Entity, With<Projectile>>,
    enemy_query: Query<Entity, With<Enemy>>,
    sensor_query: Query<Entity, With<Sensor>>,
    portal_query: Query<&Portal>,
    player_query: Query<Entity, With<Player>>,
    npc_query: Query<Entity, With<NPC>>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        let mut found_match = false;

        // Perform collision from e1 -> e2 and e2 -> e1 so both have the others damage applied
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            if let (Ok(_), Ok(_)) = (npc_query.get(e1), player_query.get(e2)) {
                found_match = true;
                break;
            }

            // Checks if one of the entities is a projectile and one is an enemy
            if let Ok(projectile_entity) = projectile_query.get(e1) {
                if let Ok(enemy_entity) = enemy_query.get(e2) {
                    projectile_hit_event.send(ProjectileHitEvent {
                        projectile: projectile_entity,
                        enemy: enemy_entity,
                    });
                    found_match = true;
                    // Once we find a match we go to the next collision
                    break;
                }
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
                    found_match = true;
                    break;
                }
            }
            if let (Ok(_), Ok(_)) = (sensor_query.get(e1), player_query.get(e2)) {
                debug!("Found collision with sensor, no-op");
                found_match = true;
                break;
            }
        }

        if !found_match {
            error!("Unexpected collision was not handled")
        }
    }
}
