use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    collision::EnemyCollidesWithPlayer,
    enemy::{CollisionDamage, Enemy},
    map::{
        components::Portal,
        events::{StartRunEvent, WarpZoneEnterEvent},
    },
    player::Player,
    projectile::{events::ProjectileHitEvent, Projectile},
};

/**
 * Main collision loop in game, dispatches various collisions to other systems via events
 */
pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events_started: EventReader<CollisionStarted>,
    mut projectile_hit_event: EventWriter<ProjectileHitEvent>,
    mut warpzone_enter_event_writer: EventWriter<WarpZoneEnterEvent>,
    mut run_start_portal_event_writer: EventWriter<StartRunEvent>,
    projectile_query: Query<Entity, With<Projectile>>,
    enemy_query: Query<(Entity, &CollisionDamage), With<Enemy>>,
    portal_query: Query<&Portal>,
    player_query: Query<Entity, With<Player>>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        // Perform collision from e1 -> e2 and e2 -> e1 so both have the others damage applied
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            // Checks if one of the entities is a projectile and one is an enemy
            if let Ok(projectile_entity) = projectile_query.get(e1) {
                if let Ok((enemy_entity, _)) = enemy_query.get(e2) {
                    debug!(
                        "Enemy {} collided with projectile {}",
                        enemy_entity, projectile_entity
                    );
                    projectile_hit_event.send(ProjectileHitEvent {
                        projectile: projectile_entity,
                        enemy: enemy_entity,
                    });
                    // Once we find a match we go to the next collision
                    break;
                }
            }

            if let Ok(_player_entity) = player_query.get(e1) {
                warn!("Player collision start");
                if let Ok((enemy_entity, collision_damage)) = enemy_query.get(e2) {
                    warn!("Enemy Collided With Player");
                    commands.trigger_targets(
                        EnemyCollidesWithPlayer {
                            collision_damage: collision_damage.clone(),
                        },
                        enemy_entity,
                    );
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
                    break;
                }
            }
        }
    }
}
