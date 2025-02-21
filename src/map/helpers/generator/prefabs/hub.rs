use std::collections::HashMap;

use bevy::{
    math::{Rect, Vec2},
    transform::components::Transform,
};
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::{
    components::{EnvironmentalMapCollider, EnvironmentalType, MarkerType, TileType},
    helpers::generator::{utils::calculate_center_rect, MapData},
};

use super::prefab::Prefab;

const PLAYER_SPAWN_Y_OFFSET: f32 = 5.0;
const LEVEL_EXIT_Y_OFFSET: f32 = 23.0;
const NPC_OFFSET: f32 = 5.0;
const HUB_WIDTH: u32 = 25; // Reduced size for better control
const HUB_HEIGHT: u32 = 25;
const ENTRANCE_WIDTH: i32 = 5;

pub struct Hub;

impl Prefab for Hub {
    fn build(&self, map_data: &mut MapData) -> Option<Rect> {
        let hub_size = TilemapSize {
            x: HUB_WIDTH,
            y: HUB_HEIGHT,
        };
        let hub_bounds = calculate_center_rect(map_data.size, hub_size);

        add_hub_cobblestone(map_data, &hub_bounds);
        add_hub_walls(map_data, &hub_bounds);
        add_hub_entrance(map_data, &hub_bounds);

        Some(hub_bounds)
    }

    fn get_markers(&self, bounds: &Rect) -> HashMap<MarkerType, Vec<Vec2>> {
        let mut markers: HashMap<MarkerType, Vec<Vec2>> = HashMap::new();
        let center_of_hub = bounds.center();

        // Generate player spawn
        let player_spawn = Vec2::new(center_of_hub.x, bounds.min.y + PLAYER_SPAWN_Y_OFFSET);
        markers.insert(MarkerType::PlayerSpawns, vec![player_spawn]);

        // Generate level exit
        let level_exit = Vec2::new(center_of_hub.x, bounds.min.y + LEVEL_EXIT_Y_OFFSET);
        markers.insert(MarkerType::LevelExits, vec![level_exit]);

        // Generate NPC positions
        let npc_positions = vec![
            Vec2::new(center_of_hub.x + NPC_OFFSET, center_of_hub.y + NPC_OFFSET),
            Vec2::new(center_of_hub.x - NPC_OFFSET, center_of_hub.y - NPC_OFFSET),
            Vec2::new(center_of_hub.x + NPC_OFFSET, center_of_hub.y - NPC_OFFSET),
        ];
        markers.insert(MarkerType::NPCSpawns, npc_positions);

        markers
    }
}

fn add_hub_cobblestone(map_data: &mut MapData, bounds: &Rect) {
    for x in bounds.min.x as i32..bounds.max.x as i32 {
        for y in bounds.min.y as i32..bounds.max.y as i32 {
            map_data.tiles[x as usize][y as usize] = TileType::Cobblestone;
        }
    }
}

fn add_hub_walls(map_data: &mut MapData, bounds: &Rect) {
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

fn add_hub_entrance(map_data: &mut MapData, bounds: &Rect) {
    let entrance_width = ENTRANCE_WIDTH;
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
