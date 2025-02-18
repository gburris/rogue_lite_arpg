use crate::map::components::{EnvironmentalMapCollider, EnvironmentalType, TileType};
use bevy::{
    math::{Rect, Vec2},
    transform::components::Transform,
};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

pub fn create_hub(map_size: TilemapSize, hub_size: TilemapSize) -> MapData {
    let mut map_data = MapData::new_with_grass(map_size);

    // Add exterior walls with colliders
    add_exterior_walls(&mut map_data, map_size);

    // Calculate hub bounds
    let hub_center = Vec2::new((map_size.x / 2) as f32, (map_size.y / 2) as f32);
    let hub_bounds =
        Rect::from_center_size(hub_center, Vec2::new(hub_size.x as f32, hub_size.y as f32));

    add_hub_cobblestone(&mut map_data, &hub_bounds);
    add_hub_walls(&mut map_data, hub_bounds);
    add_hub_entrance(&mut map_data, hub_bounds);

    map_data
}

pub struct MapData {
    pub tiles: Vec<Vec<TileType>>,
    pub colliders: Vec<EnvironmentalMapCollider>,
}

impl MapData {
    fn new(size: TilemapSize) -> Self {
        Self {
            tiles: vec![vec![TileType::Ground; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
        }
    }
    fn new_with_grass(size: TilemapSize) -> Self {
        Self {
            tiles: vec![vec![TileType::Grass; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
        }
    }
    fn add_wall_collider(&mut self, start: (u32, u32), is_horizontal: bool, length: u32) {
        let start_pos = Vec2::new(start.0 as f32, start.1 as f32);
        let length = length as f32;

        let (width, height) = if is_horizontal {
            (length, 1.0)
        } else {
            (1.0, length)
        };

        let collider_pos = if is_horizontal {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + 0.5)
        } else {
            Vec2::new(start_pos.x + 0.5, start_pos.y + (height / 2.0))
        };

        self.colliders.push(EnvironmentalMapCollider {
            collider_type: EnvironmentalType::Wall,
            transform: Transform::from_xyz(collider_pos.x, collider_pos.y, 1.0),
            width,
            height,
        });
    }
}

pub fn create_map_with_exterior_walls_and_dead_zones(
    map_size: TilemapSize,
    should_make_zones: bool,
) -> MapData {
    let mut map_data = MapData::new(map_size);

    // Add dead zones if requested
    if should_make_zones {
        add_dead_zones(&mut map_data, map_size);
    }

    // Add exterior walls and their colliders
    add_exterior_walls(&mut map_data, map_size);

    map_data
}

fn add_dead_zones(map_data: &mut MapData, map_size: TilemapSize) {
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

fn add_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {
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

fn calculate_num_dead_zones(area: u32) -> u32 {
    if area < 625 {
        return 0;
    }
    ((area as f32 / 2500.0).ceil() as u32).min(10)
}

fn add_hub_cobblestone(map_data: &mut MapData, bounds: &Rect) {
    for x in bounds.min.x as i32..bounds.max.x as i32 {
        for y in bounds.min.y as i32..bounds.max.y as i32 {
            if x >= 0
                && y >= 0
                && x < map_data.tiles.len() as i32
                && y < map_data.tiles[0].len() as i32
            {
                map_data.tiles[x as usize][y as usize] = TileType::Cobblestone;
            }
        }
    }
}

fn add_hub_walls(map_data: &mut MapData, bounds: Rect) {
    let min_x = bounds.min.x as i32;
    let max_x = bounds.max.x as i32;
    let min_y = bounds.min.y as i32;
    let max_y = bounds.max.y as i32;
    let wall_thickness = 3;

    // Add horizontal walls (top and bottom)
    for wall_layer in 0..wall_thickness {
        // Top wall
        if min_y + wall_layer < max_y {
            let start_x = min_x;
            let y = min_y + wall_layer;
            let mut current_length = 0;
            let mut wall_start = start_x;

            for x in start_x..max_x {
                map_data.tiles[x as usize][y as usize] = TileType::Wall;
                if current_length == 0 {
                    wall_start = x;
                }
                current_length += 1;
            }
            if current_length > 0 {
                map_data.add_wall_collider(
                    (wall_start as u32, y as u32),
                    true,
                    current_length as u32,
                );
            }
        }

        // Bottom wall
        if max_y - wall_layer - 1 >= min_y {
            let start_x = min_x;
            let y = max_y - wall_layer - 1;
            let mut current_length = 0;
            let mut wall_start = start_x;

            for x in start_x..max_x {
                map_data.tiles[x as usize][y as usize] = TileType::Wall;
                if current_length == 0 {
                    wall_start = x;
                }
                current_length += 1;
            }
            if current_length > 0 {
                map_data.add_wall_collider(
                    (wall_start as u32, y as u32),
                    true,
                    current_length as u32,
                );
            }
        }
    }

    // Add vertical walls (left and right)
    for wall_layer in 0..wall_thickness {
        // Left wall
        if min_x + wall_layer < max_x {
            let x = min_x + wall_layer;
            let mut current_length = 0;
            let mut wall_start = min_y;

            for y in min_y..max_y {
                map_data.tiles[x as usize][y as usize] = TileType::Wall;
                if current_length == 0 {
                    wall_start = y;
                }
                current_length += 1;
            }
            if current_length > 0 {
                map_data.add_wall_collider(
                    (x as u32, wall_start as u32),
                    false,
                    current_length as u32,
                );
            }
        }

        // Right wall
        if max_x - wall_layer - 1 >= min_x {
            let x = max_x - wall_layer - 1;
            let mut current_length = 0;
            let mut wall_start = min_y;

            for y in min_y..max_y {
                map_data.tiles[x as usize][y as usize] = TileType::Wall;
                if current_length == 0 {
                    wall_start = y;
                }
                current_length += 1;
            }
            if current_length > 0 {
                map_data.add_wall_collider(
                    (x as u32, wall_start as u32),
                    false,
                    current_length as u32,
                );
            }
        }
    }
}

fn add_hub_entrance(map_data: &mut MapData, bounds: Rect) {
    let entrance_width = 5;
    let entrance_x_start = (bounds.min.x as i32 + bounds.max.x as i32) / 2 - entrance_width / 2;

    // Create the entrance path
    let y_range_start = bounds.min.y as i32 - 5;
    let y_range_end = bounds.min.y as i32 + 5;

    // Find colliders that need to be modified
    let mut colliders_to_remove = Vec::new();
    let mut new_colliders = Vec::new();

    for (i, collider) in map_data.colliders.iter().enumerate() {
        let collider_x = collider.transform.translation.x;
        let collider_y = collider.transform.translation.y;

        // Check if this collider intersects with our entrance
        if collider_y >= y_range_start as f32
            && collider_y <= y_range_end as f32
            && collider.width > entrance_width as f32
        {
            // For horizontal colliders that cross the entrance
            if collider.width > collider.height {
                let left_edge = collider_x - collider.width / 2.0;
                let right_edge = collider_x + collider.width / 2.0;

                // If the collider spans our entrance area
                if left_edge < entrance_x_start as f32
                    && right_edge > (entrance_x_start + entrance_width) as f32
                {
                    colliders_to_remove.push(i);

                    // Create left side collider
                    let left_width = entrance_x_start as f32 - left_edge;
                    if left_width > 0.0 {
                        new_colliders.push(EnvironmentalMapCollider {
                            collider_type: EnvironmentalType::Wall,
                            transform: Transform::from_xyz(
                                left_edge + left_width / 2.0,
                                collider_y,
                                1.0,
                            ),
                            width: left_width,
                            height: collider.height,
                        });
                    }

                    // Create right side collider
                    let right_width = right_edge - (entrance_x_start + entrance_width) as f32;
                    if right_width > 0.0 {
                        new_colliders.push(EnvironmentalMapCollider {
                            collider_type: EnvironmentalType::Wall,
                            transform: Transform::from_xyz(
                                (entrance_x_start + entrance_width) as f32 + right_width / 2.0,
                                collider_y,
                                1.0,
                            ),
                            width: right_width,
                            height: collider.height,
                        });
                    }
                }
            }
        }
    }

    // Remove old colliders and add new ones
    for index in colliders_to_remove.iter().rev() {
        map_data.colliders.remove(*index);
    }
    map_data.colliders.extend(new_colliders);

    // Add wooden path tiles
    for x in entrance_x_start..(entrance_x_start + entrance_width) {
        for y in y_range_start..y_range_end {
            if x >= 0
                && y >= 0
                && x < map_data.tiles.len() as i32
                && y < map_data.tiles[0].len() as i32
            {
                map_data.tiles[x as usize][y as usize] = TileType::Wood;
            }
        }
    }
}
