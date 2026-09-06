use bevy::{
    log::warn,
    math::{IVec2, UVec2, Vec2},
};
use rand::RngExt;
use std::collections::HashMap;

use crate::world::map::{
    MarkerType, TileType,
    generation::MapData,
    prefabs::Prefab,
    space::{TileGrid, TileRect},
};

pub struct EmptySquare;

impl Prefab for EmptySquare {
    fn build(&self, map_data: &mut MapData) -> Option<TileRect> {
        if let Some(bounds) = find_dead_zone_position(&map_data.grid) {
            add_dead_zone_structure(map_data, &bounds);
            Some(bounds)
        } else {
            warn!("No valid dead zone position was found");
            None
        }
    }

    fn get_markers(&self, _bounds: &TileRect) -> HashMap<MarkerType, Vec<Vec2>> {
        HashMap::new()
    }
}

fn find_dead_zone_position(map: &TileGrid) -> Option<TileRect> {
    let mut rng = rand::rng();
    let max_attempts = 50;
    let size = rng.random_range(3..=10) as i32;
    let min_distance = 4;

    for _ in 0..max_attempts {
        let map_size = map.map_size();
        let max_x = (map_size.x as i32 - size - min_distance).max(0);
        let max_y = (map_size.y as i32 - size - min_distance).max(0);
        let min_x = min_distance;
        let min_y = min_distance;

        if max_x <= min_x || max_y <= min_y {
            continue;
        }

        let start_x = rng.random_range(min_x..max_x);
        let start_y = rng.random_range(min_y..max_y);

        let bounds = TileRect {
            min: IVec2::new(start_x, start_y),
            size: UVec2::new(size as u32, size as u32),
        };

        if can_place_dead_zone(map, &bounds) {
            return Some(bounds);
        }
    }
    None
}

fn can_place_dead_zone(grid: &TileGrid, bounds: &TileRect) -> bool {
    let padded_bounds = bounds.padded(2);

    if !grid.contains_rect(&padded_bounds) {
        return false;
    }

    grid.rect_iter(&padded_bounds)
        .all(|(_, tile_type)| tile_type.can_spawn())
}

fn add_dead_zone_structure(map_data: &mut MapData, bounds: &TileRect) {
    let start_x = bounds.min.x as i32;
    let start_y = bounds.min.y as i32;
    let size = (bounds.size.x as i32).max(3);

    // Fill interior with DeadZone tiles
    for x in start_x..(start_x + size) {
        for y in start_y..(start_y + size) {
            map_data.grid.set(x, y, TileType::DeadZone);
        }
    }

    // Add walls and colliders around perimeter
    let wall_size = size + 2;
    if start_x > 0 && start_y > 0 {
        // Top wall
        for x in (start_x - 1)..=(start_x + size) {
            map_data.grid.set(x, start_y - 1, TileType::Wall);
        }
        map_data.add_wall_collider(
            (start_x as u32 - 1, start_y as u32 - 1),
            true,
            wall_size as u32,
        );

        // Bottom wall
        for x in (start_x - 1)..=(start_x + size) {
            map_data.grid.set(x, start_y + size, TileType::Wall);
        }
        map_data.add_wall_collider(
            (start_x as u32 - 1, start_y as u32 + size as u32),
            true,
            wall_size as u32,
        );

        // Left wall
        for y in (start_y - 1)..=(start_y + size) {
            map_data.grid.set(start_x - 1, y, TileType::Wall);
        }
        map_data.add_wall_collider(
            (start_x as u32 - 1, start_y as u32 - 1),
            false,
            wall_size as u32,
        );

        // Right wall
        for y in (start_y - 1)..=(start_y + size) {
            map_data.grid.set(start_x + size, y, TileType::Wall);
        }
        map_data.add_wall_collider(
            (start_x as u32 + size as u32, start_y as u32 - 1),
            false,
            wall_size as u32,
        );
    }
}
