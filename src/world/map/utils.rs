use bevy::math::{UVec2, Vec2};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use crate::world::map::{TileType, space::TileGrid};

#[derive(Debug)]
enum MapOrientation {
    Horizontal,
    Vertical,
    Square,
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
    grid: &TileGrid,
    x_range: std::ops::Range<f32>,
    rng: &mut ChaCha8Rng,
) -> Option<Vec2> {
    let x_start = grid.map_size().x as f32 * x_range.start;
    let x_end = grid.map_size().x as f32 * x_range.end;
    for _ in 0..100 {
        let x = rng.random_range(x_start..x_end);
        let y = rng.random_range(1.0..grid.map_size().y as f32 - 1.0); // Avoid spawning in exterior walls

        if is_position_valid(grid, x as i32, y as i32) {
            return Some(Vec2::new(x, y));
        }
    }
    None
}

pub fn find_multiple_positions(
    grid: &TileGrid,
    x_range: std::ops::Range<f32>,
    count: u32,
    rng: &mut ChaCha8Rng,
) -> Vec<Vec2> {
    let mut positions = Vec::new();
    let mut attempts = 0;

    while positions.len() < count as usize && attempts < 100 {
        if let Some(pos) = find_valid_position(grid, x_range.clone(), rng)
            && !positions.iter().any(|p: &Vec2| p.distance(pos) < 5.0)
        {
            positions.push(pos);
        }
        attempts += 1;
    }

    positions
}

const INVALID_SPAWN_TILES: [TileType; 2] = [TileType::Wall, TileType::DeadZone];

fn is_position_valid(grid: &TileGrid, x: i32, y: i32) -> bool {
    !INVALID_SPAWN_TILES.contains(&grid.get(x, y).unwrap_or(TileType::DeadZone))
}

fn determine_map_orientation(map_size: UVec2) -> MapOrientation {
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
    map_size: UVec2,
    num_exits: u32,
    rng: &mut ChaCha8Rng,
) -> (Vec<Vec2>, Vec<Vec2>) {
    let player_spawn = match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // For horizontal maps: left to right
            let player_x = 1.0; // One tile in from left wall
            let player_y = rng.random_range(1.0..(map_size.y - 1) as f32);
            vec![Vec2::new(player_x, player_y)]
        }
        MapOrientation::Vertical => {
            // For vertical maps: top to bottom
            let player_y = map_size.y as f32 - 2.0; // One tile down from top wall
            let player_x = rng.random_range(1.0..(map_size.x - 1) as f32);
            vec![Vec2::new(player_x, player_y)]
        }
        MapOrientation::Square => {
            // For square maps: default to left to right
            let player_x = 1.0;
            let player_y = rng.random_range(1.0..(map_size.y - 1) as f32);
            vec![Vec2::new(player_x, player_y)]
        }
    };

    let exits = match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // Two exits on the right side
            let exit_x = map_size.x as f32 - 1.0;
            let exit_y1 = rng.random_range(1.0..(map_size.y / 2) as f32);
            let exit_y2 = rng.random_range((map_size.y / 2) as f32..(map_size.y - 1) as f32);
            vec![Vec2::new(exit_x, exit_y1), Vec2::new(exit_x, exit_y2)]
        }
        MapOrientation::Vertical => {
            // Two exits at the bottom
            let exit_y = 1.0;
            let exit_x1 = rng.random_range(1.0..(map_size.x / 2) as f32);
            let exit_x2 = rng.random_range((map_size.x / 2) as f32..(map_size.x - 1) as f32);
            vec![Vec2::new(exit_x1, exit_y), Vec2::new(exit_x2, exit_y)]
        }
        MapOrientation::Square => {
            // Two exits on the right side for square maps
            let exit_x = map_size.x as f32 - 1.0;
            let exit_y1 = rng.random_range(1.0..(map_size.y / 2) as f32);
            let exit_y2 = rng.random_range((map_size.y / 2) as f32..(map_size.y - 1) as f32);
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
