use bevy::{
    log::warn,
    math::{Rect, Vec2},
};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;
use std::collections::HashMap;

use crate::map::{
    components::{MarkerType, TileType},
    helpers::generator::{
        utils::{calculate_center_rect, is_position_valid},
        MapData,
    },
};

use super::prefab::Prefab;
pub struct Temple;

impl Prefab for Temple {
    fn build(&self, map_data: &mut MapData) -> Option<Rect> {
        if let Some(bounds) = find_temple_position(&map_data.tiles, map_data.size) {
            add_temple_structure(map_data, &bounds);
            Some(bounds)
        } else {
            warn!("No valid temple position was found");
            None
        }
    }

    fn get_markers(&self, bounds: &Rect) -> HashMap<MarkerType, Vec<Vec2>> {
        let mut markers = HashMap::new();

        let chest_pos = Vec2::new(
            bounds.min.x + TEMPLE_WIDTH as f32 / 2.0,
            bounds.min.y + TEMPLE_HEIGHT as f32 / 2.0,
        );

        markers.insert(MarkerType::ChestSpawns, vec![chest_pos]);

        markers
    }
}

const TEMPLE_WIDTH: u32 = 7;
const TEMPLE_HEIGHT: u32 = 7;
const ENTRANCE_WIDTH: u32 = 3;

fn find_temple_position(map: &Vec<Vec<TileType>>, map_size: TilemapSize) -> Option<Rect> {
    let mut rng = rand::thread_rng();
    let max_attempts = 100;

    let temple_size = TilemapSize {
        x: TEMPLE_WIDTH,
        y: TEMPLE_HEIGHT,
    };

    for _ in 0..max_attempts {
        let offset_x = rng.gen_range(-(map_size.x as i32 / 4)..(map_size.x as i32 / 4));
        let offset_y = rng.gen_range(-(map_size.y as i32 / 4)..(map_size.y as i32 / 4));

        let base_bounds = calculate_center_rect(map_size, temple_size);
        let min_x = base_bounds.min.x + offset_x as f32;
        let min_y = base_bounds.min.y + offset_y as f32;
        let max_x = min_x + TEMPLE_WIDTH as f32;
        let max_y = min_y + TEMPLE_HEIGHT as f32;
        let bounds = Rect::new(min_x, min_y, max_x, max_y);

        if can_place_temple(map, &bounds) {
            return Some(bounds);
        }
    }
    None
}

fn can_place_temple(map: &Vec<Vec<TileType>>, bounds: &Rect) -> bool {
    for x in (bounds.min.x as i32 - 1)..=(bounds.min.x + bounds.width()) as i32 + 1 {
        for y in (bounds.min.y as i32 - 1)..=(bounds.min.y + bounds.height()) as i32 + 1 {
            if x >= map.len() as i32 || y >= map[0].len() as i32 || x < 0 || y < 0 {
                return false;
            }
            if !is_position_valid(map, x as u32, y as u32) {
                return false;
            }
        }
    }
    true
}

fn add_temple_structure(map_data: &mut MapData, bounds: &Rect) {
    let start_x = bounds.min.x as u32;
    let start_y = bounds.min.y as u32;

    // Clear the area first
    for x in start_x..start_x + TEMPLE_WIDTH {
        for y in start_y..start_y + TEMPLE_HEIGHT {
            if x < map_data.tiles.len() as u32 && y < map_data.tiles[0].len() as u32 {
                map_data.tiles[x as usize][y as usize] = TileType::Cobblestone;
            }
        }
    }

    map_data.add_wall_collider((start_x, start_y), true, TEMPLE_WIDTH);

    let entrance_start = start_x + (TEMPLE_WIDTH - ENTRANCE_WIDTH) / 2;
    if entrance_start > start_x {
        map_data.add_wall_collider(
            (start_x, start_y + TEMPLE_HEIGHT - 1),
            true,
            entrance_start - start_x,
        );
    }

    let after_entrance = entrance_start + ENTRANCE_WIDTH;
    if after_entrance < start_x + TEMPLE_WIDTH {
        map_data.add_wall_collider(
            (after_entrance, start_y + TEMPLE_HEIGHT - 1),
            true,
            (start_x + TEMPLE_WIDTH) - after_entrance,
        );
    }

    map_data.add_wall_collider((start_x, start_y), false, TEMPLE_HEIGHT);
    map_data.add_wall_collider((start_x + TEMPLE_WIDTH - 1, start_y), false, TEMPLE_HEIGHT);

    for x in start_x..start_x + TEMPLE_WIDTH {
        for y in start_y..start_y + TEMPLE_HEIGHT {
            if x < map_data.tiles.len() as u32 && y < map_data.tiles[0].len() as u32 {
                // Set walls around the perimeter
                if x == start_x
                    || x == start_x + TEMPLE_WIDTH - 1
                    || y == start_y
                    || (y == start_y + TEMPLE_HEIGHT - 1
                        && (x < entrance_start || x >= entrance_start + ENTRANCE_WIDTH))
                {
                    map_data.tiles[x as usize][y as usize] = TileType::Wall;
                }
            }
        }
    }
}
