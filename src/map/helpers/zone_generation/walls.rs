use crate::map::components::TileType;

use bevy_ecs_tilemap::map::TilemapSize;

use super::map_data::MapData;

pub fn add_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {
    // Add horizontal walls and colliders
    add_horizontal_exterior_walls(map_data, map_size);
    // Add vertical walls and colliders
    add_vertical_exterior_walls(map_data, map_size);
}

fn add_horizontal_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {
    let mut top_wall_start = 0;
    let mut bottom_wall_start = 0;
    let mut current_length = 0;

    for x in 0..map_size.x as usize {
        // Process top wall
        if map_data.tiles[x][0] != TileType::DeadZone {
            map_data.tiles[x][0] = TileType::Wall;
            if current_length == 0 {
                top_wall_start = x;
            }
            current_length += 1;
        } else if current_length > 0 {
            // Create collider for the wall section we just finished
            map_data.add_wall_collider((top_wall_start as u32, 0), true, current_length as u32);
            current_length = 0;
        }
    }

    // Add any remaining top wall section
    if current_length > 0 {
        map_data.add_wall_collider((top_wall_start as u32, 0), true, current_length as u32);
    }

    // Reset for bottom wall
    current_length = 0;

    for x in 0..map_size.x as usize {
        // Process bottom wall
        if map_data.tiles[x][map_size.y as usize - 1] != TileType::DeadZone {
            map_data.tiles[x][map_size.y as usize - 1] = TileType::Wall;
            if current_length == 0 {
                bottom_wall_start = x;
            }
            current_length += 1;
        } else if current_length > 0 {
            // Create collider for the wall section we just finished
            map_data.add_wall_collider(
                (bottom_wall_start as u32, (map_size.y - 1)),
                true,
                current_length as u32,
            );
            current_length = 0;
        }
    }

    // Add any remaining bottom wall section
    if current_length > 0 {
        map_data.add_wall_collider(
            (bottom_wall_start as u32, (map_size.y - 1)),
            true,
            current_length as u32,
        );
    }
}

fn add_vertical_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {
    let mut left_wall_start = 0;
    let mut right_wall_start = 0;
    let mut current_length = 0;

    for y in 0..map_size.y as usize {
        // Process left wall
        if map_data.tiles[0][y] != TileType::DeadZone {
            map_data.tiles[0][y] = TileType::Wall;
            if current_length == 0 {
                left_wall_start = y;
            }
            current_length += 1;
        } else if current_length > 0 {
            // Create collider for the wall section we just finished
            map_data.add_wall_collider((0, left_wall_start as u32), false, current_length as u32);
            current_length = 0;
        }
    }

    // Add any remaining left wall section
    if current_length > 0 {
        map_data.add_wall_collider((0, left_wall_start as u32), false, current_length as u32);
    }

    // Reset for right wall
    current_length = 0;

    for y in 0..map_size.y as usize {
        // Process right wall
        if map_data.tiles[map_size.x as usize - 1][y] != TileType::DeadZone {
            map_data.tiles[map_size.x as usize - 1][y] = TileType::Wall;
            if current_length == 0 {
                right_wall_start = y;
            }
            current_length += 1;
        } else if current_length > 0 {
            // Create collider for the wall section we just finished
            map_data.add_wall_collider(
                ((map_size.x - 1), right_wall_start as u32),
                false,
                current_length as u32,
            );
            current_length = 0;
        }
    }

    // Add any remaining right wall section
    if current_length > 0 {
        map_data.add_wall_collider(
            ((map_size.x - 1), right_wall_start as u32),
            false,
            current_length as u32,
        );
    }
}
