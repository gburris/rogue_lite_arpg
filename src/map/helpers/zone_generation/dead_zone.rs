use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

use crate::map::components::TileType;

use super::{map_data::MapData, utils::calculate_num_dead_zones};

pub fn add_dead_zones(map_data: &mut MapData, map_size: TilemapSize) {
    let area = map_size.x * map_size.y;
    let num_dead_zones = calculate_num_dead_zones(area);

    for _ in 0..num_dead_zones {
        create_dead_zone(map_data, map_size);
    }
}

fn create_dead_zone(map_data: &mut MapData, map_size: TilemapSize) {
    let mut rng = rand::thread_rng();
    let size = rng.gen_range(3..=10);
    let min_distance: u32 = 4;

    // Calculate valid position ranges
    let max_x = (map_size.x - size - min_distance) as u32;
    let max_y = (map_size.y - size - min_distance) as u32;
    let min_x = min_distance;
    let min_y = min_distance;

    if max_x <= min_x || max_y <= min_y {
        return;
    }

    let start_x = rng.gen_range(min_x..max_x) as usize;
    let start_y = rng.gen_range(min_y..max_y) as usize;

    // Validate location
    if !is_valid_dead_zone_location(&map_data.tiles, map_size, start_x, start_y, size) {
        return;
    }

    // Add dead zone walls and their colliders
    add_dead_zone_walls(map_data, start_x, start_y, size);

    // Fill in dead zone interior
    for x in start_x..(start_x + size as usize) {
        for y in start_y..(start_y + size as usize) {
            map_data.tiles[x][y] = TileType::DeadZone;
        }
    }
}

fn is_valid_dead_zone_location(
    tiles: &[Vec<TileType>],
    map_size: TilemapSize,
    start_x: usize,
    start_y: usize,
    size: u32,
) -> bool {
    for x in (start_x.saturating_sub(2))..=(start_x + size as usize + 2) {
        for y in (start_y.saturating_sub(2))..=(start_y + size as usize + 2) {
            if x >= map_size.x as usize || y >= map_size.y as usize {
                continue;
            }
            if tiles[x][y] == TileType::DeadZone || tiles[x][y] == TileType::Wall {
                return false;
            }
        }
    }
    true
}

fn add_dead_zone_walls(map_data: &mut MapData, start_x: usize, start_y: usize, size: u32) {
    // Add horizontal walls and colliders
    for x in (start_x - 1)..=(start_x + size as usize) {
        // Top wall
        map_data.tiles[x][start_y - 1] = TileType::Wall;
        // Bottom wall
        map_data.tiles[x][start_y + size as usize] = TileType::Wall;
    }

    // Add horizontal colliders
    map_data.add_wall_collider((start_x as u32 - 1, start_y as u32 - 1), true, size + 2);
    map_data.add_wall_collider((start_x as u32 - 1, start_y as u32 + size), true, size + 2);

    // Add vertical walls and colliders
    for y in (start_y - 1)..=(start_y + size as usize) {
        // Left wall
        map_data.tiles[start_x - 1][y] = TileType::Wall;
        // Right wall
        map_data.tiles[start_x + size as usize][y] = TileType::Wall;
    }

    // Add vertical colliders
    map_data.add_wall_collider((start_x as u32 - 1, start_y as u32 - 1), false, size + 2);
    map_data.add_wall_collider((start_x as u32 + size, start_y as u32 - 1), false, size + 2);
}
