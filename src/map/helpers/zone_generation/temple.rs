use bevy::{
    log::warn,
    math::{Rect, Vec2},
    transform::components::Transform,
};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;
use std::collections::HashMap;

use super::{
    map_data::MapData,
    utils::{calculate_center_rect, is_position_valid},
};
use crate::map::components::{EnvironmentalMapCollider, EnvironmentalType, MarkerType, TileType};

const TEMPLE_WIDTH: u32 = 9;
const TEMPLE_HEIGHT: u32 = 6;
const ENTRANCE_WIDTH: u32 = 4;

fn find_temple_position(map: &Vec<Vec<TileType>>, map_size: TilemapSize) -> Option<Rect> {
    let mut rng = rand::thread_rng();
    let max_attempts = 100;

    let temple_size = TilemapSize {
        x: TEMPLE_WIDTH,
        y: TEMPLE_HEIGHT,
    };

    for _ in 0..max_attempts {
        // Instead of generating raw coordinates, generate an offset from the center
        let offset_x = rng.gen_range(-(map_size.x as i32 / 4)..(map_size.x as i32 / 4));
        let offset_y = rng.gen_range(-(map_size.y as i32 / 4)..(map_size.y as i32 / 4));

        // Get base bounds from center rect
        let base_bounds = calculate_center_rect(map_size, temple_size);

        // Apply the random offset
        let bounds = Rect::new(
            base_bounds.min.x + offset_x as f32,
            base_bounds.min.y + offset_y as f32,
            TEMPLE_WIDTH as f32,
            TEMPLE_HEIGHT as f32,
        );

        if can_place_temple(map, &bounds) {
            return Some(bounds);
        }
    }
    None
}

fn can_place_temple(map: &Vec<Vec<TileType>>, bounds: &Rect) -> bool {
    for x in bounds.min.x as i32..(bounds.min.x + bounds.width()) as i32 {
        for y in bounds.min.y as i32..(bounds.min.y + bounds.height()) as i32 {
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

pub fn build_temple(map_data: &mut MapData) -> Option<Rect> {
    if let Some(bounds) = find_temple_position(&map_data.tiles, map_data.size) {
        add_temple_walls(map_data, &bounds);
        Some(bounds)
    } else {
        warn!("No valid temple position was found");
        None
    }
}

fn add_temple_walls(map_data: &mut MapData, bounds: &Rect) {
    let min_x = bounds.min.x as i32;
    let max_x = (bounds.min.x + bounds.width()) as i32;
    let min_y = bounds.min.y as i32;
    let max_y = (bounds.min.y + bounds.height()) as i32;

    // Calculate entrance position
    let entrance_start = min_x + (bounds.width() as i32 - ENTRANCE_WIDTH as i32) / 2;

    // 1. Top wall (full width)
    add_horizontal_wall(map_data, min_x, min_y, bounds.width() as u32);

    // 2. Left wall (full height)
    add_vertical_wall(map_data, min_x, min_y, bounds.height() as u32);

    // 3. Right wall (full height)
    add_vertical_wall(map_data, max_x - 1, min_y, bounds.height() as u32);

    // 4. Bottom left wall (partial)
    let left_entrance_width = (entrance_start - min_x) as u32;
    if left_entrance_width > 0 {
        add_horizontal_wall(map_data, min_x, max_y - 1, left_entrance_width);
    }

    // 5. Bottom right wall (partial)
    let right_entrance_width = (max_x - (entrance_start + ENTRANCE_WIDTH as i32)) as u32;
    if right_entrance_width > 0 {
        add_horizontal_wall(
            map_data,
            entrance_start + ENTRANCE_WIDTH as i32,
            max_y - 1,
            right_entrance_width,
        );
    }
}

fn add_horizontal_wall(map_data: &mut MapData, start_x: i32, y: i32, width: u32) {
    // Add wall tiles
    for x in start_x..start_x + width as i32 {
        if x >= 0 && x < map_data.tiles.len() as i32 && y >= 0 && y < map_data.tiles[0].len() as i32
        {
            map_data.tiles[x as usize][y as usize] = TileType::Wall;
        }
    }

    // Add single collider for the wall section
    map_data.colliders.push(EnvironmentalMapCollider {
        collider_type: EnvironmentalType::Wall,
        transform: Transform::from_xyz(start_x as f32 + width as f32 / 2.0, y as f32, 1.0),
        width: width as f32,
        height: 1.0,
    });
}

fn add_vertical_wall(map_data: &mut MapData, x: i32, start_y: i32, height: u32) {
    // Add wall tiles
    for y in start_y..start_y + height as i32 {
        if x >= 0 && x < map_data.tiles.len() as i32 && y >= 0 && y < map_data.tiles[0].len() as i32
        {
            map_data.tiles[x as usize][y as usize] = TileType::Wall;
        }
    }

    // Add single collider for the wall section
    map_data.colliders.push(EnvironmentalMapCollider {
        collider_type: EnvironmentalType::Wall,
        transform: Transform::from_xyz(x as f32, start_y as f32 + height as f32 / 2.0, 1.0),
        width: 1.0,
        height: height as f32,
    });
}

pub fn get_temple_markers(bounds: &Rect) -> HashMap<MarkerType, Vec<Vec2>> {
    let mut markers = HashMap::new();
    let center = bounds.center();

    markers.insert(
        MarkerType::EnemySpawns,
        vec![
            Vec2::new(center.x, center.y),       // Center
            Vec2::new(center.x - 2.0, center.y), // Left
            Vec2::new(center.x + 2.0, center.y), // Right
            Vec2::new(center.x, center.y - 2.0), // Bottom
            Vec2::new(center.x, center.y + 2.0), // Top
        ],
    );

    markers
}
