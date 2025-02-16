use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;
use std::collections::HashMap;

use crate::map::components::{InstanceAssets, MapLayout, MapMarkers, MarkerType, TileType};

pub fn generate_instance_layout(instance_assets: &Res<InstanceAssets>) -> MapLayout {
    let mut rng = rand::thread_rng();

    // Randomly select instance type
    let instance_type = if rng.gen_bool(0.9) {
        instance_assets.instance_config.get("Swamp").unwrap()
    } else {
        instance_assets.instance_config.get("TreasureRoom").unwrap()
    };

    // Generate size based on instance ranges
    let size_x = rng.gen_range(instance_type.size_x_range.0..=instance_type.size_x_range.1) as u32;
    let size_y = rng.gen_range(instance_type.size_y_range.0..=instance_type.size_y_range.1) as u32;
    let map_size = TilemapSize {
        x: size_x,
        y: size_y,
    };

    // Generate number of enemies and chests
    let num_enemies = rng.gen_range(
        instance_type.number_of_enemies_range.0..=instance_type.number_of_enemies_range.1,
    ) as u32;
    let num_chests =
        rng.gen_range(instance_type.chest_range.0..=instance_type.chest_range.1) as u32;

    // Generate tiles
    let tiles = super::create_tile_layout::create_map_with_exterior_walls_and_dead_zones(
        map_size,
        instance_type.dead_zone_squares,
    );

    // Generate markers
    let markers = generate_instance_markers(&tiles, map_size, num_enemies, num_chests);

    // TODO: Generate "pre-fabs"
    //Prefabs are groupings of markers and tiles.
    //Roll dice to set up a prefab
    //Find valid location (At least 25x25 square of ground tiles somewhere)
    //Add wall tiles, special ground tiles, enemy markers, chest markers, NPC markers etc

    // Generate colliders
    let environmental_colliders =
        super::add_colliders::add_environmental_colliders_to_zone(&tiles, map_size);

    MapLayout {
        size: map_size,
        tiles,
        markers,
        environmental_colliders,
    }
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

fn generate_entrance_exit_positions(map_size: TilemapSize) -> (Vec<Vec2>, Vec<Vec2>) {
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
    (player_spawn, exits)
}

fn generate_instance_markers(
    map: &Vec<Vec<TileType>>,
    map_size: TilemapSize,
    enemy_count: u32,
    chest_count: u32,
) -> MapMarkers {
    let mut markers = HashMap::new();

    // Generate entrance and exit positions based on map orientation
    let (player_pos, exit_positions) = generate_entrance_exit_positions(map_size);
    markers.insert(MarkerType::PlayerSpawns, player_pos);
    markers.insert(MarkerType::LevelExits, exit_positions);

    // Generate enemy spawns in the middle section
    let enemy_positions = find_multiple_positions(map, map_size, 0.3..0.7, enemy_count);
    if !enemy_positions.is_empty() {
        markers.insert(MarkerType::EnemySpawns, enemy_positions);
    }

    // Generate chest spawns throughout the map
    let chest_positions = find_multiple_positions(map, map_size, 0.2..0.8, chest_count);
    if !chest_positions.is_empty() {
        markers.insert(MarkerType::ChestSpawns, chest_positions);
    }

    MapMarkers { markers }
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
