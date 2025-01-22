use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

use crate::map::components::{MapLayout, MapMarker, TileType};

pub fn generate_map_layout(map_size: TilemapSize) -> MapLayout {
    // First generate the base physical map
    let mut tiles = create_empty_map(map_size);

    // Add walls and obstacles
    generate_walls(&mut tiles, map_size);
    generate_water_bodies(&mut tiles, map_size);

    // Generate markers after physical layout is done
    let markers = generate_markers(&tiles, map_size);

    MapLayout { tiles, markers }
}

fn create_empty_map(map_size: TilemapSize) -> Vec<Vec<TileType>> {
    vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize]
}

fn generate_walls(map: &mut Vec<Vec<TileType>>, map_size: TilemapSize) {
    let num_walls = (map_size.x as f32 * map_size.y as f32 * 0.002) as i32;

    for _ in 0..num_walls {
        if let Some((start_x, start_y, is_horizontal, length)) = generate_wall_parameters(map_size)
        {
            if can_place_wall(map, map_size, start_x, start_y, is_horizontal, length) {
                place_wall(map, start_x, start_y, is_horizontal, length);
            }
        }
    }
}

fn generate_water_bodies(map: &mut Vec<Vec<TileType>>, map_size: TilemapSize) {
    let mut rng = rand::thread_rng();
    let num_water_bodies = (map_size.x as f32 * map_size.y as f32 * 0.001) as i32;

    for _ in 0..num_water_bodies {
        let x = rng.gen_range(5..(map_size.x - 5) as i32);
        let y = rng.gen_range(5..(map_size.y - 5) as i32);
        let radius = rng.gen_range(2..5);

        place_water_body(map, map_size, x, y, radius);
    }
}

fn place_water_body(
    map: &mut Vec<Vec<TileType>>,
    map_size: TilemapSize,
    center_x: i32,
    center_y: i32,
    radius: i32,
) {
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let x = center_x + dx;
                let y = center_y + dy;

                if x >= 0 && x < map_size.x as i32 && y >= 0 && y < map_size.y as i32 {
                    map[x as usize][y as usize] = TileType::Water;
                }
            }
        }
    }
}

fn generate_markers(map: &Vec<Vec<TileType>>, map_size: TilemapSize) -> Vec<MapMarker> {
    let mut markers = Vec::new();

    // Generate player spawn in the left third of the map
    if let Some(spawn_pos) = find_valid_position(map, map_size, 0.0..0.3) {
        markers.push(MapMarker::PlayerSpawn(spawn_pos));
    }

    // Generate exit in the right third of the map
    if let Some(exit_pos) = find_valid_position(map, map_size, 0.7..1.0) {
        markers.push(MapMarker::LevelExit(exit_pos));
    }

    // Generate enemy spawns in the middle section
    let enemy_positions = find_multiple_positions(map, map_size, 0.3..0.7, 5);
    if !enemy_positions.is_empty() {
        markers.push(MapMarker::EnemySpawns(enemy_positions));
    }

    // Generate chest spawns throughout the map
    let chest_positions = find_multiple_positions(map, map_size, 0.2..0.8, 3);
    if !chest_positions.is_empty() {
        markers.push(MapMarker::ChestSpawns(chest_positions));
    }

    // Generate boss spawn in the far right if map is large enough
    if map_size.x >= 50 {
        if let Some(boss_pos) = find_valid_position(map, map_size, 0.8..0.9) {
            markers.push(MapMarker::BossSpawns(vec![boss_pos]));
        }
    }

    markers
}

fn find_valid_position(
    map: &Vec<Vec<TileType>>,
    map_size: TilemapSize,
    x_range: std::ops::Range<f32>,
) -> Option<Vec2> {
    let mut rng = rand::thread_rng();
    let x_start = (map_size.x as f32 * x_range.start) as u32;
    let x_end = (map_size.x as f32 * x_range.end) as u32;

    for _ in 0..100 {
        let x = rng.gen_range(x_start..x_end);
        let y = rng.gen_range(0..map_size.y);

        if is_position_valid(map, x, y) {
            return Some(Vec2::new(x as f32, y as f32));
        }
    }
    None
}

fn find_multiple_positions(
    map: &Vec<Vec<TileType>>,
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

fn is_position_valid(map: &Vec<Vec<TileType>>, x: u32, y: u32) -> bool {
    map[x as usize][y as usize] == TileType::Ground
}

fn generate_wall_parameters(map_size: TilemapSize) -> Option<(i32, i32, bool, i32)> {
    let mut rng = rand::thread_rng();
    let wall_length = rng.gen_range(8..20);
    let start_x = rng.gen_range(5..(map_size.x as i32 - wall_length));
    let start_y = rng.gen_range(5..(map_size.y as i32 - wall_length));
    let is_horizontal = rng.gen_bool(0.5);
    Some((start_x, start_y, is_horizontal, wall_length))
}

fn can_place_wall(
    map: &Vec<Vec<TileType>>,
    map_size: TilemapSize,
    start_x: i32,
    start_y: i32,
    is_horizontal: bool,
    wall_length: i32,
) -> bool {
    let padding = 2;
    for i in 0..wall_length {
        let (x, y) = if is_horizontal {
            (start_x + i, start_y)
        } else {
            (start_x, start_y + i)
        };

        if !is_area_clear(map, map_size, x, y, padding) {
            return false;
        }
    }
    true
}

fn is_area_clear(
    map: &Vec<Vec<TileType>>,
    map_size: TilemapSize,
    x: i32,
    y: i32,
    padding: i32,
) -> bool {
    for dx in -padding..=padding {
        for dy in -padding..=padding {
            let check_x = x + dx;
            let check_y = y + dy;
            if check_x >= 0
                && check_x < map_size.x as i32
                && check_y >= 0
                && check_y < map_size.y as i32
                && map[check_x as usize][check_y as usize] != TileType::Ground
            {
                return false;
            }
        }
    }
    true
}

fn place_wall(
    map: &mut Vec<Vec<TileType>>,
    start_x: i32,
    start_y: i32,
    is_horizontal: bool,
    wall_length: i32,
) {
    for i in 0..wall_length {
        if is_horizontal {
            map[(start_x + i) as usize][start_y as usize] = TileType::Wall;
        } else {
            map[start_x as usize][(start_y + i) as usize] = TileType::Wall;
        }
    }
}
