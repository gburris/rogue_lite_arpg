
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileTextureIndex};
use rand::Rng;

use crate::components::{Enemy, Level, Player, Projectile, WarpZone};
use crate::events::WarpZoneEnterEvent;
use crate::resources::MapBounds;

pub fn handle_warpzone_enter(
    mut commands: Commands,
    mut events: EventReader<WarpZoneEnterEvent>,
    mut transform_queries: ParamSet<(
        Query<(&mut WarpZone, &mut Transform)>,
        Query<&mut Transform, With<Player>>,
    )>,
    mut tilemap_query: Query<(&TileStorage, &TilemapSize)>,
    mut tile_query: Query<&mut TileTextureIndex>,
    enemies: Query<Entity, With<Enemy>>,
    projectiles: Query<Entity, With<Projectile>>,
    map_bounds: Res<MapBounds>,
) {
    for _event in events.read() {
        // First, handle warpzone updates
        let (next_level, should_continue) = {
            let mut warpzone_query = transform_queries.p0();
            if let Ok((mut warp_zone, mut warp_transform)) = warpzone_query.get_single_mut() {
                // Update the warpzone's level
                let current_level = warp_zone.level.clone();
                let next_level = Level::next_level(current_level);
                warp_zone.level = next_level.clone();

                // Move warpzone to a random position on the right side of the map
                let mut rng = rand::thread_rng();
                let new_y = rng.gen_range(-200.0..200.0);
                warp_transform.translation.x = map_bounds.max_x - 100.0;
                warp_transform.translation.y = new_y;

                (Some(next_level), true)
            } else {
                (None, false)
            }
        };

        if !should_continue {
            continue;
        }

        let next_level = next_level.unwrap();

        // Then handle player position
        {
            let mut player_query = transform_queries.p1();
            if let Ok(mut transform) = player_query.get_single_mut() {
                transform.translation.x = map_bounds.min_x;
                transform.translation.y = 0.0;
            }
        }

        // Recolor all the map tiles
        for (tile_storage, map_size) in tilemap_query.iter_mut() {
            for x in 0..map_size.x {
                for y in 0..map_size.y {
                    let tile_pos = TilePos { x, y };
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
                            tile_texture.0 = Level::to_int(&next_level);
                        }
                    }
                }
            }
        }

        // Despawn all enemies
        for enemy in enemies.iter() {
            commands.entity(enemy).try_despawn();
        }

        // Despawn all projectiles
        for projectile in projectiles.iter() {
            commands.entity(projectile).try_despawn();
        }

        println!("Progressing to level: {:?}", Level::to_int(&next_level));
    }
}
