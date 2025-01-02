use crate::components::WarpZone;
use crate::events::WarpZoneEnterEvent;
use crate::player::Player;
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn check_warpzone_collision(
    mut collision_events_started: EventReader<CollisionStarted>,
    mut warpzone_enter_event_writer: EventWriter<WarpZoneEnterEvent>,
    warpzone_query: Query<Entity, With<WarpZone>>,
    player_query: Query<Entity, With<Player>>,
) {
    for CollisionStarted(e1, e2) in collision_events_started.read() {
        for (e1, e2) in [(*e1, *e2), (*e2, *e1)] {
            // Checks if one of the entities is a projectile and one is an enemy
            if let Ok(warpzone_entity) = warpzone_query.get(e1) {
                if let Ok(_player_entity) = player_query.get(e2) {
                    warpzone_enter_event_writer.send(WarpZoneEnterEvent {
                        warp_zone: warpzone_entity,
                    });
                }
            }
        }
    }
}
