use bevy::{
    log::warn,
    math::{IVec2, UVec2, Vec2},
};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::RngExt;
use std::collections::HashMap;

use crate::{
    prelude::Prefab,
    world::map::{
        MarkerType, TileType,
        map_data::MapData,
        space::{TileGrid, TileRect},
        utils::{calculate_center_rect, is_position_valid},
    },
};

pub struct Temple;

impl Prefab for Temple {
    fn build(&self, map_data: &mut MapData) -> Option<TileRect> {
        if let Some(bounds) = find_temple_position(&map_data.grid) {
            add_temple_structure(map_data, &bounds);
            Some(bounds)
        } else {
            warn!("No valid temple position was found");
            None
        }
    }

    fn get_markers(&self, bounds: &TileRect) -> HashMap<MarkerType, Vec<Vec2>> {
        let mut markers = HashMap::new();

        let chest_pos = Vec2::new(
            bounds.min.x as f32 + TEMPLE_WIDTH as f32 / 2.0,
            bounds.min.y as f32 + TEMPLE_HEIGHT as f32 / 2.0,
        );

        markers.insert(MarkerType::ChestSpawns, vec![chest_pos]);

        markers
    }
}

const TEMPLE_WIDTH: u32 = 7;
const TEMPLE_HEIGHT: u32 = 7;
const ENTRANCE_WIDTH: u32 = 3;

fn find_temple_position(grid: &TileGrid) -> Option<TileRect> {
    let mut rng = rand::rng();
    let max_attempts = 100;

    let temple_size = TilemapSize {
        x: TEMPLE_WIDTH,
        y: TEMPLE_HEIGHT,
    };

    for _ in 0..max_attempts {
        let offset_x =
            rng.random_range(-(grid.map_size().x as i32 / 4)..(grid.map_size().x as i32 / 4));
        let offset_y =
            rng.random_range(-(grid.map_size().y as i32 / 4)..(grid.map_size().y as i32 / 4));

        let base_bounds = calculate_center_rect(grid.map_size().into(), temple_size);
        let min_x = base_bounds.min.x + offset_x as f32;
        let min_y = base_bounds.min.y + offset_y as f32;

        let bounds = TileRect {
            min: IVec2::new(min_x as i32, min_y as i32),
            size: UVec2::new(TEMPLE_WIDTH, TEMPLE_HEIGHT),
        };

        if can_place_temple(grid, &bounds) {
            return Some(bounds);
        }
    }
    None
}

fn can_place_temple(grid: &TileGrid, bounds: &TileRect) -> bool {
    for x in (bounds.min.x as i32 - 1)..=(bounds.min.x + bounds.size.x as i32) as i32 + 1 {
        for y in (bounds.min.y as i32 - 1)..=(bounds.min.y + bounds.size.y as i32) as i32 + 1 {
            if x >= grid.map_size().x as i32 || y >= grid.map_size().y as i32 || x < 0 || y < 0 {
                return false;
            }
            if !is_position_valid(grid, x, y) {
                return false;
            }
        }
    }
    true
}

fn add_temple_structure(map_data: &mut MapData, bounds: &TileRect) {
    let start_x = bounds.min.x as u32;
    let start_y = bounds.min.y as u32;

    map_data.grid.fill_rect(bounds, TileType::Cobblestone);

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

    let grid_size = map_data.grid.map_size();
    for x in start_x..start_x + TEMPLE_WIDTH {
        for y in start_y..start_y + TEMPLE_HEIGHT {
            // Bounds check using map_size()
            if x < grid_size.x && y < grid_size.y {
                let is_wall = x == start_x
                    || x == start_x + TEMPLE_WIDTH - 1
                    || y == start_y
                    || (y == start_y + TEMPLE_HEIGHT - 1
                        && (x < entrance_start || x >= entrance_start + ENTRANCE_WIDTH));
                if is_wall {
                    let _ = map_data.grid.set(x as i32, y as i32, TileType::Wall);
                }
            }
        }
    }
}
