use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::map::TilemapSize;
//use noise::{NoiseFn, Perlin}; //TODO: Make the water on this map noisey and beautiful

use crate::map::{
    components::{MapLayout, TileType},
    MapMarkers, MarkerType,
};

pub fn generate_hub_map(size: TilemapSize) -> MapLayout {
    let mut tiles = create_grass_map(size);

    let hub_size = TilemapSize { x: 75, y: 75 }; // Change (1): Hub size reduced to 75x75
    let hub_center = Vec2::new((size.x / 2) as f32, (size.y / 2) as f32);
    let hub_bounds =
        Rect::from_center_size(hub_center, Vec2::new(hub_size.x as f32, hub_size.y as f32));

    add_walls(&mut tiles, hub_bounds);
    add_moat(&mut tiles, hub_bounds);
    add_wall_entrance(&mut tiles, hub_bounds); // Change (4): Add wall entrance

    let markers = generate_hub_markers(hub_bounds);

    MapLayout {
        size,
        tiles,
        markers,
    }
}

fn create_grass_map(map_size: TilemapSize) -> Vec<Vec<TileType>> {
    vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize]
}

fn add_walls(map: &mut Vec<Vec<TileType>>, bounds: Rect) {
    for x in bounds.min.x as i32..bounds.max.x as i32 {
        for y in bounds.min.y as i32..bounds.max.y as i32 {
            let is_wall = x < bounds.min.x as i32 + 3 // Change (2): Walls are 3 tiles thick
                || x >= bounds.max.x as i32 - 3
                || y < bounds.min.y as i32 + 3
                || y >= bounds.max.y as i32 - 3;

            if is_wall {
                map[x as usize][y as usize] = TileType::Wall;
            }
        }
    }
}

fn add_moat(map: &mut Vec<Vec<TileType>>, bounds: Rect) {
    let moat_offset = 3; // Distance from wall

    for x in (bounds.min.x as i32 - moat_offset)..(bounds.max.x as i32 + moat_offset) {
        for y in (bounds.min.y as i32 - moat_offset)..(bounds.max.y as i32 + moat_offset) {
            let is_moat = (x < bounds.min.x as i32 || x >= bounds.max.x as i32)
                || (y < bounds.min.y as i32 || y >= bounds.max.y as i32);

            if is_moat && x >= 0 && y >= 0 && x < map.len() as i32 && y < map[0].len() as i32 {
                map[x as usize][y as usize] = TileType::Water;
            }
        }
    }
}

fn add_wall_entrance(map: &mut Vec<Vec<TileType>>, bounds: Rect) {
    let entrance_width = 20; // Change (4): Entrance width is 20 tiles
    let entrance_x_start = (bounds.min.x as i32 + bounds.max.x as i32) / 2 - entrance_width / 2;

    for x in entrance_x_start..(entrance_x_start + entrance_width) {
        for y in bounds.min.y as i32..(bounds.min.y as i32 + 10) {
            map[x as usize][y as usize] = TileType::Ground;
        }
    }
}

fn generate_hub_markers(bounds: Rect) -> MapMarkers {
    let mut single_markers = HashMap::new();

    // Spawn player near the entrance
    let player_spawn = Vec2::new(
        (bounds.min.x + bounds.max.x) as f32 / 2.0,
        bounds.min.y as f32 + 5.0,
    );
    single_markers.insert(MarkerType::PlayerSpawn, player_spawn);

    MapMarkers {
        single_markers,
        multi_markers: HashMap::new(),
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
