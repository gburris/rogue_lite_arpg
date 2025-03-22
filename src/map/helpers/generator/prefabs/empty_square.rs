use bevy::{
    log::warn,
    math::{Rect, Vec2},
};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;
use std::collections::HashMap;

use crate::map::{
    components::{MarkerType, TileType},
    helpers::generator::MapData,
};

use super::prefab::Prefab;

pub struct EmptySquare;

impl Prefab for EmptySquare {
    fn build(&self, map_data: &mut MapData) -> Option<Rect> {
        if let Some(bounds) = find_dead_zone_position(&map_data.tiles, map_data.size) {
            add_dead_zone_structure(map_data, &bounds);
            Some(bounds)
        } else {
            warn!("No valid dead zone position was found");
            None
        }
    }

    fn get_markers(&self, _bounds: &Rect) -> HashMap<MarkerType, Vec<Vec2>> {
        HashMap::new()
    }
}

fn find_dead_zone_position(map: &[Vec<TileType>], map_size: TilemapSize) -> Option<Rect> {
    let mut rng = rand::thread_rng();
    let max_attempts = 50;
    let size = rng.gen_range(3..=10) as f32;
    let min_distance = 4.0;

    for _ in 0..max_attempts {
        let max_x = (map_size.x as f32 - size - min_distance).max(0.0);
        let max_y = (map_size.y as f32 - size - min_distance).max(0.0);
        let min_x = min_distance;
        let min_y = min_distance;

        if max_x <= min_x || max_y <= min_y {
            continue;
        }

        let start_x = rng.gen_range(min_x..max_x);
        let start_y = rng.gen_range(min_y..max_y);
        let bounds = Rect::new(start_x, start_y, start_x + size, start_y + size);

        if can_place_dead_zone(map, &bounds) {
            return Some(bounds);
        }
    }
    None
}

fn can_place_dead_zone(map: &[Vec<TileType>], bounds: &Rect) -> bool {
    let buffer = 2;
    for x in (bounds.min.x as isize - buffer)..=(bounds.max.x as isize + buffer) {
        for y in (bounds.min.y as isize - buffer)..=(bounds.max.y as isize + buffer) {
            if x >= map.len() as isize || y >= map[0].len() as isize || x < 0 || y < 0 {
                continue;
            }
            let tile = map[x as usize][y as usize];
            if tile == TileType::DeadZone || tile == TileType::Wall {
                return false;
            }
        }
    }
    true
}

fn add_dead_zone_structure(map_data: &mut MapData, bounds: &Rect) {
    let start_x = bounds.min.x as usize; // Changed to usize
    let start_y = bounds.min.y as usize; // Changed to usize
    let size = (bounds.width() as usize).max(3); // Changed to usize

    // Fill interior with DeadZone tiles
    for x in start_x..(start_x + size) {
        for y in start_y..(start_y + size) {
            if x < map_data.tiles.len() && y < map_data.tiles[0].len() {
                map_data.tiles[x][y] = TileType::DeadZone;
            }
        }
    }

    // Add walls and colliders around perimeter
    let wall_size = size + 2;
    if start_x > 0 && start_y > 0 {
        // Top wall
        for x in (start_x - 1)..(start_x + size + 1) {
            if x < map_data.tiles.len() {
                map_data.tiles[x][start_y - 1] = TileType::Wall;
            }
        }
        map_data.add_wall_collider(
            (start_x as u32 - 1, start_y as u32 - 1),
            true,
            wall_size as u32,
        );

        // Bottom wall
        for x in (start_x - 1)..(start_x + size + 1) {
            if x < map_data.tiles.len() && (start_y + size) < map_data.tiles[0].len() {
                map_data.tiles[x][start_y + size] = TileType::Wall;
            }
        }
        map_data.add_wall_collider(
            (start_x as u32 - 1, start_y as u32 + size as u32),
            true,
            wall_size as u32,
        );

        // Left wall
        for y in (start_y - 1)..(start_y + size + 1) {
            if y < map_data.tiles[0].len() {
                map_data.tiles[start_x - 1][y] = TileType::Wall;
            }
        }
        map_data.add_wall_collider(
            (start_x as u32 - 1, start_y as u32 - 1),
            false,
            wall_size as u32,
        );

        // Right wall
        for y in (start_y - 1)..(start_y + size + 1) {
            if (start_x + size) < map_data.tiles.len() && y < map_data.tiles[0].len() {
                map_data.tiles[start_x + size][y] = TileType::Wall;
            }
        }
        map_data.add_wall_collider(
            (start_x as u32 + size as u32, start_y as u32 - 1),
            false,
            wall_size as u32,
        );
    }
}
