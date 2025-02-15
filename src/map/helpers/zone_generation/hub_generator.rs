use std::collections::HashMap;

use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::{
    components::{MapLayout, TileType},
    MapMarkers, MarkerType, MultiMarkerType,
};

use super::{collider_generator::generate_environmental_colliders, create_map_with_exterior_walls};

pub fn generate_hub_layout() -> MapLayout {
    let size = TilemapSize { x: 100, y: 100 };
    let mut tiles = create_map_with_exterior_walls(size);

    let hub_size = TilemapSize { x: 25, y: 25 };
    let hub_center = Vec2::new((size.x / 2) as f32, (size.y / 2) as f32);
    let hub_bounds =
        Rect::from_center_size(hub_center, Vec2::new(hub_size.x as f32, hub_size.y as f32));

    add_cobblestone(&mut tiles, &hub_bounds);

    add_walls(&mut tiles, hub_bounds);
    add_wall_entrance(&mut tiles, hub_bounds);

    let markers: MapMarkers = generate_hub_markers(hub_bounds);
    let environmental_colliders = generate_environmental_colliders(&tiles, size);

    MapLayout {
        size,
        tiles,
        markers,
        environmental_colliders,
    }
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

fn generate_hub_markers(bounds: Rect) -> MapMarkers {
    let mut single_markers = HashMap::new();
    let mut multi_markers = HashMap::new();
    let center_of_hub = Vec2::new(
        (bounds.min.x + bounds.max.x) / 2.0,
        (bounds.min.y + bounds.max.y) / 2.0,
    );

    let player_spawn = Vec2::new(center_of_hub.x, bounds.min.y as f32 + 5.0);
    single_markers.insert(MarkerType::PlayerSpawn, player_spawn);

    let level_exit_spawn = Vec2::new(center_of_hub.x, bounds.min.y as f32 + 23.0);

    single_markers.insert(MarkerType::LevelExit, level_exit_spawn);
    let npc_positiions = vec![
        Vec2::new(center_of_hub.x + 5.0, center_of_hub.y + 5.0),
        Vec2::new(center_of_hub.x - 5.0, center_of_hub.y + -5.0),
        Vec2::new(center_of_hub.x + 5.0, center_of_hub.y + -5.0),
    ];
    multi_markers.insert(MultiMarkerType::NPCSpawns, npc_positiions);

    MapMarkers {
        single_markers,
        multi_markers,
    }
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }
}
