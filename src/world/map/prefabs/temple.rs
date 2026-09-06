use bevy::{
    log::warn,
    math::{IVec2, Vec2},
};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::RngExt;
use std::collections::HashMap;

use crate::{
    prelude::Prefab,
    world::map::{
        MarkerType, TileType,
        generation::MapData,
        space::{TileGrid, TileRect},
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

    let base_bounds = TileRect::new_centered_in(grid.map_size(), temple_size.into())?;

    for _ in 0..max_attempts {
        let offset_x =
            rng.random_range(-(grid.map_size().x as i32 / 4)..(grid.map_size().x as i32 / 4));
        let offset_y =
            rng.random_range(-(grid.map_size().y as i32 / 4)..(grid.map_size().y as i32 / 4));

        let bounds = base_bounds.offset_from(IVec2::new(offset_x, offset_y));

        if grid
            .rect_iter(&bounds)
            .all(|(_, tile_type)| tile_type.can_spawn())
        {
            return Some(bounds);
        }
    }
    None
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
