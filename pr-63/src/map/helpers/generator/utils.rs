use bevy::math::{Rect, Vec2};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

use crate::map::components::TileType;

#[derive(Debug)]
enum MapOrientation {
    Horizontal,
    Vertical,
    Square,
}

pub fn calculate_center_rect(map_size: TilemapSize, size: TilemapSize) -> Rect {
    let center = Vec2::new((map_size.x / 2) as f32, (map_size.y / 2) as f32);
    Rect::from_center_size(center, Vec2::new(size.x as f32, size.y as f32))
}

pub fn calculate_wall_dimensions(is_horizontal: bool, length: f32) -> (f32, f32) {
    if is_horizontal {
        (length, 1.0)
    } else {
        (1.0, length)
    }
}

pub fn calculate_collider_position(
    start_pos: Vec2,
    width: f32,
    height: f32,
    is_horizontal: bool,
) -> Vec2 {
    if is_horizontal {
        Vec2::new(start_pos.x + (width / 2.0), start_pos.y + 0.5)
    } else {
        Vec2::new(start_pos.x + 0.5, start_pos.y + (height / 2.0))
    }
}

pub fn find_valid_position(
    map: &[Vec<TileType>],
    map_size: TilemapSize,
    x_range: std::ops::Range<f32>,
) -> Option<Vec2> {
    let mut rng = rand::thread_rng();
    let x_start = (map_size.x as f32 * x_range.start) as u32;
    let x_end = (map_size.x as f32 * x_range.end) as u32;

    for _ in 0..100 {
        let x = rng.gen_range(x_start..x_end);
        let y = rng.gen_range(1..map_size.y - 1); // Avoid spawning in exterior walls

        if is_position_valid(map, x, y) {
            return Some(Vec2::new(x as f32, y as f32));
        }
    }
    None
}

pub fn find_multiple_positions(
    map: &[Vec<TileType>],
    map_size: TilemapSize,
    x_range: std::ops::Range<f32>,
    count: u32,
) -> Vec<Vec2> {
    let mut positions = Vec::new();
    let mut attempts = 0;

    while positions.len() < count as usize && attempts < 100 {
        if let Some(pos) = find_valid_position(map, map_size, x_range.clone()) {
            if !positions.iter().any(|p: &Vec2| p.distance(pos) < 5.0) {
                positions.push(pos);
            }
        }
        attempts += 1;
    }

    positions
}

const INVALID_SPAWN_TILES: [TileType; 2] = [TileType::Wall, TileType::DeadZone];

pub fn is_position_valid(map: &[Vec<TileType>], x: u32, y: u32) -> bool {
    let tile = &map[x as usize][y as usize];
    !INVALID_SPAWN_TILES.contains(tile)
}

fn determine_map_orientation(map_size: TilemapSize) -> MapOrientation {
    let aspect_ratio = map_size.x as f32 / map_size.y as f32;

    if (aspect_ratio - 1.0).abs() < 0.1 {
        MapOrientation::Square
    } else if aspect_ratio > 1.0 {
        MapOrientation::Horizontal
    } else {
        MapOrientation::Vertical
    }
}

pub fn generate_entrance_exit_positions(
    map_size: TilemapSize,
    num_exits: u32,
) -> (Vec<Vec2>, Vec<Vec2>) {
    let mut rng = rand::thread_rng();

    let player_spawn = match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // For horizontal maps: left to right
            let player_x = 1.0; // One tile in from left wall
            let player_y = rng.gen_range(1..map_size.y - 1) as f32;
            vec![Vec2::new(player_x, player_y)]
        }
        MapOrientation::Vertical => {
            // For vertical maps: top to bottom
            let player_y = map_size.y as f32 - 2.0; // One tile down from top wall
            let player_x = rng.gen_range(1..map_size.x - 1) as f32;
            vec![Vec2::new(player_x, player_y)]
        }
        MapOrientation::Square => {
            // For square maps: default to left to right
            let player_x = 1.0;
            let player_y = rng.gen_range(1..map_size.y - 1) as f32;
            vec![Vec2::new(player_x, player_y)]
        }
    };

    let exits = match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // Two exits on the right side
            let exit_x = map_size.x as f32 - 1.0;
            let exit_y1 = rng.gen_range(1..map_size.y / 2) as f32;
            let exit_y2 = rng.gen_range(map_size.y / 2..map_size.y - 1) as f32;
            vec![Vec2::new(exit_x, exit_y1), Vec2::new(exit_x, exit_y2)]
        }
        MapOrientation::Vertical => {
            // Two exits at the bottom
            let exit_y = 1.0;
            let exit_x1 = rng.gen_range(1..map_size.x / 2) as f32;
            let exit_x2 = rng.gen_range(map_size.x / 2..map_size.x - 1) as f32;
            vec![Vec2::new(exit_x1, exit_y), Vec2::new(exit_x2, exit_y)]
        }
        MapOrientation::Square => {
            // Two exits on the right side for square maps
            let exit_x = map_size.x as f32 - 1.0;
            let exit_y1 = rng.gen_range(1..map_size.y / 2) as f32;
            let exit_y2 = rng.gen_range(map_size.y / 2..map_size.y - 1) as f32;
            vec![Vec2::new(exit_x, exit_y1), Vec2::new(exit_x, exit_y2)]
        }
    };
    if num_exits == 0 {
        (player_spawn, [].to_vec())
    } else if num_exits == 1 {
        (player_spawn, vec![exits[0]])
    } else {
        (player_spawn, exits)
    }
}
