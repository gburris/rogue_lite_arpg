use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;
use std::collections::HashMap;

use crate::map::{
    components::{MapLayout, TileType},
    InstanceAssets, MapMarkers, MarkerType, MultiMarkerType,
};

pub fn generate_map_layout(instance_assets: &Res<InstanceAssets>) -> MapLayout {
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
    let tiles = super::tile_generator::create_map_with_exterior_walls_and_dead_zones(
        map_size,
        instance_type.dead_zone_squares,
    );

    // Generate markers
    let markers = generate_markers(&tiles, map_size, num_enemies, num_chests);

    // Generate "pre-fabs"
    //Prefabs are groupings of markers and tiles.
    //Roll dice to set up a prefab
    //Find valid location (At least 25x25 square of ground tiles somewhere)
    //Add wall tiles, special ground tiles, enemy markers, chest markers, NPC markers etc

    // Generate colliders
    let environmental_colliders =
        super::collider_generator::generate_environmental_colliders(&tiles, map_size);

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
    chest_count: u32,
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
    let chest_positions = find_multiple_positions(map, map_size, 0.2..0.8, chest_count);
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
