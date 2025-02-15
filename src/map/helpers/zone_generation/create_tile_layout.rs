use crate::map::components::TileType;
use bevy::math::{Rect, Vec2};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

pub fn create_hub(map_size: TilemapSize, hub_size: TilemapSize) -> Vec<Vec<TileType>> {
    let mut map = vec![vec![TileType::Grass; map_size.y as usize]; map_size.x as usize];

    // Add top and bottom walls
    for x in 0..map_size.x as usize {
        map[x][0] = TileType::Wall;
        map[x][map_size.y as usize - 1] = TileType::Wall;
    }

    // Add left and right walls
    for y in 0..map_size.y as usize {
        map[0][y] = TileType::Wall;
        map[map_size.x as usize - 1][y] = TileType::Wall;
    }
    let hub_center = Vec2::new((map_size.x / 2) as f32, (map_size.y / 2) as f32);
    let hub_bounds =
        Rect::from_center_size(hub_center, Vec2::new(hub_size.x as f32, hub_size.y as f32));

    add_cobblestone(&mut map, &hub_bounds);
    add_walls(&mut map, hub_bounds);
    add_wall_entrance(&mut map, hub_bounds);

    map
}

pub fn create_map_with_exterior_walls_and_dead_zones(
    map_size: TilemapSize,
    should_make_zones: bool,
) -> Vec<Vec<TileType>> {
    let mut map = vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize];

    if should_make_zones {
        let area = map_size.x * map_size.y;
        let num_dead_zones = calculate_num_dead_zones(area);
        for _ in 0..num_dead_zones {
            create_square_dead_zone(&mut map, map_size);
        }
    }

    // Add walls to the border of the map only if it's not a deadzone tile
    for x in 0..map_size.x as usize {
        // Top border
        if map[x][0] != TileType::DeadZone {
            map[x][0] = TileType::Wall;
        }
        // Bottom border
        if map[x][map_size.y as usize - 1] != TileType::DeadZone {
            map[x][map_size.y as usize - 1] = TileType::Wall;
        }
    }

    for y in 0..map_size.y as usize {
        // Left border
        if map[0][y] != TileType::DeadZone {
            map[0][y] = TileType::Wall;
        }
        // Right border
        if map[map_size.x as usize - 1][y] != TileType::DeadZone {
            map[map_size.x as usize - 1][y] = TileType::Wall;
        }
    }

    map
}

fn add_cobblestone(map: &mut Vec<Vec<TileType>>, bounds: &Rect) {
    // Iterate over the integer coordinates within the bounds.
    for x in bounds.min.x as i32..bounds.max.x as i32 {
        for y in bounds.min.y as i32..bounds.max.y as i32 {
            map[x as usize][y as usize] = TileType::Cobblestone;
        }
    }
}

fn add_walls(map: &mut Vec<Vec<TileType>>, bounds: Rect) {
    for x in bounds.min.x as i32..bounds.max.x as i32 {
        for y in bounds.min.y as i32..bounds.max.y as i32 {
            let is_wall = x < bounds.min.x as i32 + 3
                || x >= bounds.max.x as i32 - 3
                || y < bounds.min.y as i32 + 3
                || y >= bounds.max.y as i32 - 3;

            if is_wall {
                map[x as usize][y as usize] = TileType::Wall;
            }
        }
    }
}

fn add_wall_entrance(map: &mut Vec<Vec<TileType>>, bounds: Rect) {
    let entrance_width = 5; // Entrance width is 20 tiles
    let entrance_x_start = (bounds.min.x as i32 + bounds.max.x as i32) / 2 - entrance_width / 2;

    // Force a solid ground bridge across the moat and into the hub
    let y_range_start = bounds.min.y as i32 - 5; // Extend the range ~20 tiles above the hub
    let y_range_end = bounds.min.y as i32 + 5; // Extend the range ~20 tiles into the hub

    for x in entrance_x_start..(entrance_x_start + entrance_width) {
        for y in y_range_start..y_range_end {
            // Ensure indices are within bounds
            if x >= 0 && y >= 0 && x < map.len() as i32 && y < map[0].len() as i32 {
                map[x as usize][y as usize] = TileType::Wood;
            }
        }
    }
}

fn create_square_dead_zone(map: &mut Vec<Vec<TileType>>, map_size: TilemapSize) {
    let mut rng = rand::thread_rng();

    // Generate random size between 3x3 and 10x10
    let size = rng.gen_range(3..=10);

    // Ensure at least 2 spaces from borders
    let min_distance: u32 = 4; // 2 spaces + 1 for the wall

    // Calculate valid position ranges
    let max_x = (map_size.x - size - min_distance) as u32;
    let max_y = (map_size.y - size - min_distance) as u32;
    let min_x = min_distance;
    let min_y = min_distance;

    if max_x <= min_x || max_y <= min_y {
        return; // Map too small for dead zone
    }

    // Generate random position (ensuring we're not too close to exterior walls)
    let start_x = rng.gen_range(min_x..max_x) as usize;
    let start_y = rng.gen_range(min_y..max_y) as usize;

    // Check if the area already contains a dead zone or is too close to one
    for x in (start_x - 2)..=(start_x + size as usize + 2) {
        for y in (start_y - 2)..=(start_y + size as usize + 2) {
            if x >= map_size.x as usize || y >= map_size.y as usize {
                continue;
            }
            if map[x][y] == TileType::DeadZone || map[x][y] == TileType::Wall {
                return; // Area too close to existing dead zone or wall
            }
        }
    }

    // Create walls around dead zone
    for x in (start_x - 1)..=(start_x + size as usize) {
        for y in (start_y - 1)..=(start_y + size as usize) {
            if x == start_x - 1
                || x == start_x + size as usize
                || y == start_y - 1
                || y == start_y + size as usize
            {
                map[x][y] = TileType::Wall;
            }
        }
    }

    // Fill in dead zone
    for x in start_x..(start_x + size as usize) {
        for y in start_y..(start_y + size as usize) {
            map[x][y] = TileType::DeadZone;
        }
    }
}

fn calculate_num_dead_zones(area: u32) -> u32 {
    // Base case: minimum map size for one dead zone is 25x25 (area 625)
    if area < 625 {
        return 0;
    }

    // Calculate number of dead zones based on area
    // This formula can be adjusted based on your needs
    let num_zones = (area as f32 / 2500.0).ceil() as u32;

    // Cap the maximum number of dead zones if needed
    num_zones.min(10) // Maximum of 10 dead zones
}
