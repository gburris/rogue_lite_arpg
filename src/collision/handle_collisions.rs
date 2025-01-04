use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    map::{
        components::{StartingPortal, WarpZone},
        events::{StartRunEvent, WarpZoneEnterEvent},
    },
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
    warpzone_query: Query<Entity, With<WarpZone>>,
    run_start_portal_query: Query<Entity, With<StartingPortal>>,
    player_query: Query<Entity, With<Player>>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        // Perform collision from e1 -> e2 and e2 -> e1 so both have the others damage applied
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            // Checks if one of the entities is a projectile and one is an enemy
            if let Ok(projectile_entity) = projectile_query.get(e1) {
                if let Ok(enemy_entity) = enemy_query.get(e2) {
                    projectile_hit_event.send(ProjectileHitEvent {
                        projectile: projectile_entity,
                        enemy: enemy_entity,
                    });
                    // Once we find a match we go to the next collision
                    break;
                }
            }
            warn!("Checking collision with starting portal");
            if let Ok(_run_start_portal_entity) = run_start_portal_query.get(e1) {
                if let Ok(_player_entity) = player_query.get(e2) {
                    warn!("Found collision with starting portal");
                    run_start_portal_event_writer.send(StartRunEvent);
                    break;
                }
            }
            // Checks if one of the entities is a warpzone and one is a player
            if let Ok(warpzone_entity) = warpzone_query.get(e1) {
                if let Ok(_player_entity) = player_query.get(e2) {
                    warpzone_enter_event_writer.send(WarpZoneEnterEvent {
                        warp_zone: warpzone_entity,
                    });
                    break;
                }
            }
        }
    }
}
