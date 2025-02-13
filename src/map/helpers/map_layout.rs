use avian2d::prelude::Collider;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;
use std::collections::HashMap;

use crate::map::{
    components::{MapLayout, TileType},
    EnvironmentalMapCollider, EnvironmentalType, InstanceAssets, MapMarkers, MarkerType,
    MultiMarkerType, WallSection,
};

pub fn generate_map_layout(size: TilemapSize, instance_assets: &Res<InstanceAssets>) -> MapLayout {
    // Generate the base map with exterior walls
    let tiles = create_map_with_exterior_walls(size);
    let instance = instance_assets.instance_config.get("Swamp").unwrap();

    // Generate markers after physical layout is done
    let markers = generate_markers(&tiles, size, instance.number_of_enemies);
    let environmental_colliders = generate_environmental_colliders(&tiles, size);

    MapLayout {
        size,
        tiles,
        markers,
        environmental_colliders,
    }
}

fn generate_environmental_colliders(
    tiles: &[Vec<TileType>],
    map_size: TilemapSize,
) -> Vec<EnvironmentalMapCollider> {
    let mut colliders = Vec::new();
    let wall_sections = find_wall_sections(tiles, map_size);

    for section in wall_sections {
        let start_pos = Vec2::new(section.start.0 as f32, section.start.1 as f32);

        let length = section.length() as f32;

        // The width/height should be in tile units
        let (width, height) = if section.is_horizontal {
            (length, 1.0)
        } else {
            (1.0, length)
        };

        let collider_pos = if section.is_horizontal {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + 0.5)
        } else {
            Vec2::new(start_pos.x + 0.5, start_pos.y + (height / 2.0))
        };

        // Create collider with half-extents (half of the full size)
        let collider = if section.is_horizontal {
            Collider::rectangle(width, 1.0)
        } else {
            Collider::rectangle(1.0, height)
        };

        colliders.push(EnvironmentalMapCollider {
            collider_type: EnvironmentalType::Wall,
            transform: Transform::from_xyz(collider_pos.x, collider_pos.y, 1.0),
            width,
            height,
        });
    }

    colliders
}

fn find_wall_sections(tiles: &[Vec<TileType>], map_size: TilemapSize) -> Vec<WallSection> {
    let mut visited = vec![vec![false; map_size.y as usize]; map_size.x as usize];
    let mut wall_sections = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            if !visited[x as usize][y as usize] && tiles[x as usize][y as usize] == TileType::Wall {
                if let Some(section) = extract_wall_section(tiles, map_size, x, y, &mut visited) {
                    wall_sections.push(section);
                }
            }
        }
    }

    wall_sections
}

fn extract_wall_section(
    tiles: &[Vec<TileType>],
    map_size: TilemapSize,
    x: u32,
    y: u32,
    visited: &mut Vec<Vec<bool>>,
) -> Option<WallSection> {
    visited[x as usize][y as usize] = true;

    // Try horizontal first
    if x + 1 < map_size.x && tiles[(x + 1) as usize][y as usize] == TileType::Wall {
        let mut section = WallSection::new((x, y), true);
        let mut current_x = x + 1;

        while current_x < map_size.x && tiles[current_x as usize][y as usize] == TileType::Wall {
            visited[current_x as usize][y as usize] = true;
            section.extend((current_x, y));
            current_x += 1;
        }

        Some(section)
    }
    // Then try vertical
    else if y + 1 < map_size.y && tiles[x as usize][(y + 1) as usize] == TileType::Wall {
        let mut section = WallSection::new((x, y), false);
        let mut current_y = y + 1;

        while current_y < map_size.y && tiles[x as usize][current_y as usize] == TileType::Wall {
            visited[x as usize][current_y as usize] = true;
            section.extend((x, current_y));
            current_y += 1;
        }

        Some(section)
    }
    // Single wall tile
    else {
        Some(WallSection::new((x, y), true))
    }
}

fn create_map_with_exterior_walls(map_size: TilemapSize) -> Vec<Vec<TileType>> {
    let mut map = vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize];

    // Add top and bottom walls
    for x in 0..map_size.x as usize {
        map[x][0] = TileType::Wall;
        map[x][map_size.y as usize - 1] = TileType::Wall;
    }

    // Add left and right walls
    for y in 0..map_size.y as usize {
        map[0][y] = TileType::Wall;
        map[map_size.x as usize - 1][y] = TileType::Wall;
    }

    map
}

#[derive(Debug)]
enum MapOrientation {
    Horizontal,
    Vertical,
    Square,
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

fn generate_entrance_exit_positions(map_size: TilemapSize) -> (Vec2, Vec2) {
    let mut rng = rand::thread_rng();

    match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // For horizontal maps: left to right
            let player_x = 1.0; // One tile in from left wall
            let exit_x = map_size.x as f32 - 1.0; // One tile in from right wall
            let player_y = rng.gen_range(1..map_size.y - 1) as f32;
            let exit_y = rng.gen_range(1..map_size.y - 1) as f32;
            (Vec2::new(player_x, player_y), Vec2::new(exit_x, exit_y))
        }
        MapOrientation::Vertical => {
            // For vertical maps: top to bottom
            let player_y = map_size.y as f32 - 2.0; // One tile down from top wall
            let exit_y = 1.0; // One tile up from bottom wall
            let player_x = rng.gen_range(1..map_size.x - 1) as f32;
            let exit_x = rng.gen_range(1..map_size.x - 1) as f32;
            (Vec2::new(player_x, player_y), Vec2::new(exit_x, exit_y))
        }
        MapOrientation::Square => {
            // For square maps: default to left to right (could be randomized if preferred)
            let player_x = 1.0;
            let exit_x = map_size.x as f32 - 1.0;
            let player_y = rng.gen_range(1..map_size.y - 1) as f32;
            let exit_y = rng.gen_range(1..map_size.y - 1) as f32;
            (Vec2::new(player_x, player_y), Vec2::new(exit_x, exit_y))
        }
    }
}

fn generate_markers(
    map: &Vec<Vec<TileType>>,
    map_size: TilemapSize,
    enemy_count: u32,
) -> MapMarkers {
    let mut single_markers = HashMap::new();
    let mut multi_markers: HashMap<MultiMarkerType, Vec<Vec2>> = HashMap::new();

    // Generate entrance and exit positions based on map orientation
    let (player_pos, exit_pos) = generate_entrance_exit_positions(map_size);
    single_markers.insert(MarkerType::PlayerSpawn, player_pos);
    single_markers.insert(MarkerType::LevelExit, exit_pos);

    // Generate enemy spawns in the middle section
    let enemy_positions = find_multiple_positions(map, map_size, 0.3..0.7, enemy_count);
    if !enemy_positions.is_empty() {
        multi_markers.insert(MultiMarkerType::EnemySpawns, enemy_positions);
    }

    // Generate chest spawns throughout the map
    let chest_positions = find_multiple_positions(map, map_size, 0.2..0.8, 3);
    if !chest_positions.is_empty() {
        multi_markers.insert(MultiMarkerType::ChestSpawns, chest_positions);
    }

    // Generate boss spawn in the far right if map is large enough
    if map_size.x >= 50 {
        if let Some(boss_pos) = find_valid_position(map, map_size, 0.8..0.9) {
            multi_markers.insert(MultiMarkerType::BossSpawns, vec![boss_pos]);
        }
    }

    MapMarkers {
        single_markers,
        multi_markers,
    }
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
        let y = rng.gen_range(1..map_size.y - 1); // Avoid spawning in exterior walls

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
