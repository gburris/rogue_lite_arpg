use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{EnemyType, PrefabType},
    world::map::{
        TileType,
        blueprint::{MapBlueprint, Placement, PlacementKind, TileGrid},
        instance::MapRecipe,
        prefabs::{build_empty_square, build_hub, build_temple},
    },
};

pub fn generate_zone(
    map_recipe: &MapRecipe,
    rng: &mut ChaCha8Rng,
    zone_seed: u64,
) -> Result<MapBlueprint> {
    let size_x = rng.random_range(map_recipe.size_x_range.0..=map_recipe.size_x_range.1) as u32;
    let size_y = rng.random_range(map_recipe.size_y_range.0..=map_recipe.size_y_range.1) as u32;

    let floor_type = map_recipe.floor_type;

    let mut blueprint = MapBlueprint::new(UVec2::new(size_x, size_y), floor_type, zone_seed);

    blueprint.tiles.fill_border(TileType::Wall);

    // Build prefabs
    for prefab in &map_recipe.prefabs {
        match prefab {
            PrefabType::Temple => build_temple(&mut blueprint),
            PrefabType::EmptySquare => build_empty_square(&mut blueprint),
            PrefabType::NpcHub => build_hub(&mut blueprint),
        }
    }

    // Determine number of enemies and chests from the recipe ranges
    let num_enemies = rng
        .random_range(map_recipe.number_of_enemies_range.0..=map_recipe.number_of_enemies_range.1)
        as u32;
    let num_chests = rng.random_range(map_recipe.chest_range.0..=map_recipe.chest_range.1) as u32;

    // Generate enemy positions (avoid walls)
    let enemy_positions = find_multiple_positions(&blueprint.tiles, 2..8, num_enemies, rng);
    for pos in enemy_positions {
        // Randomly choose an enemy type (or use a fixed set; here we pick randomly)
        let enemy_type = match rng.random_range(0..3) {
            0 => EnemyType::FireMage,
            1 => EnemyType::IceMage,
            _ => EnemyType::Warrior,
        };
        blueprint.placements.push(Placement {
            tile: pos,
            kind: PlacementKind::Enemy(enemy_type),
        });
    }

    // Generate chest positions
    let chest_positions = find_multiple_positions(&blueprint.tiles, 1..9, num_chests, rng);
    for pos in chest_positions {
        blueprint.placements.push(Placement {
            tile: pos,
            kind: PlacementKind::Chest,
        });
    }

    // Generate player spawn and exits
    let (player_positions, exit_positions) =
        generate_entrance_exit_positions(blueprint.map_size(), map_recipe.num_exits, rng);

    // Player spawn (take the first, there should be exactly one)
    if let Some(player_pos) = player_positions.first() {
        blueprint.placements.push(Placement {
            tile: *player_pos,
            kind: PlacementKind::PlayerStart,
        });
    }

    // Exits
    for pos in exit_positions {
        blueprint.placements.push(Placement {
            tile: pos,
            kind: PlacementKind::Exit,
        });
    }

    blueprint.validate().map_err(|error| error.to_string())?;
    Ok(blueprint)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MapOrientation {
    Horizontal,
    Vertical,
    Square,
}

pub fn generate_entrance_exit_positions(
    map_size: UVec2,
    num_exits: u32,
    rng: &mut ChaCha8Rng,
) -> (Vec<IVec2>, Vec<IVec2>) {
    let player_spawn = match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // For horizontal maps: left to right
            let player_x = 1; // One tile in from left wall
            let player_y = rng.random_range(1..map_size.y - 1) as i32;
            vec![IVec2::new(player_x, player_y)]
        }
        MapOrientation::Vertical => {
            // For vertical maps: top to bottom
            let player_y = map_size.y as i32 - 2; // One tile down from top wall
            let player_x = rng.random_range(1..map_size.x - 1) as i32;
            vec![IVec2::new(player_x, player_y)]
        }
        MapOrientation::Square => {
            // For square maps: default to left to right
            let player_x = 1;
            let player_y = rng.random_range(1..map_size.y - 1) as i32;
            vec![IVec2::new(player_x, player_y)]
        }
    };

    let exits = match determine_map_orientation(map_size) {
        MapOrientation::Horizontal => {
            // Two exits on the right side
            let exit_x = map_size.x as i32 - 1;
            let exit_y1 = rng.random_range(1..map_size.y / 2) as i32;
            let exit_y2 = rng.random_range(map_size.y / 2..map_size.y - 1) as i32;
            vec![IVec2::new(exit_x, exit_y1), IVec2::new(exit_x, exit_y2)]
        }
        MapOrientation::Vertical => {
            // Two exits at the bottom
            let exit_y = 1;
            let exit_x1 = rng.random_range(1..map_size.x / 2) as i32;
            let exit_x2 = rng.random_range(map_size.x / 2..map_size.x - 1) as i32;
            vec![IVec2::new(exit_x1, exit_y), IVec2::new(exit_x2, exit_y)]
        }
        MapOrientation::Square => {
            // Two exits on the right side for square maps
            let exit_x = map_size.x as i32 - 1;
            let exit_y1 = rng.random_range(1..map_size.y / 2) as i32;
            let exit_y2 = rng.random_range(map_size.y / 2..map_size.y - 1) as i32;
            vec![IVec2::new(exit_x, exit_y1), IVec2::new(exit_x, exit_y2)]
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

fn find_valid_position(
    grid: &TileGrid,
    x_range: std::ops::Range<u32>,
    rng: &mut ChaCha8Rng,
) -> Option<IVec2> {
    let x_start = grid.map_size().x * x_range.start / 10;
    let x_end = grid.map_size().x * x_range.end / 10;
    for _ in 0..100 {
        let x = rng.random_range(x_start..x_end) as i32;
        let y = rng.random_range(1..grid.map_size().y - 1) as i32;

        if is_position_valid(grid, x as i32, y as i32) {
            return Some(IVec2::new(x, y));
        }
    }
    None
}

fn find_multiple_positions(
    grid: &TileGrid,
    x_range: std::ops::Range<u32>,
    count: u32,
    rng: &mut ChaCha8Rng,
) -> Vec<IVec2> {
    let mut positions: Vec<IVec2> = Vec::new();
    let mut attempts = 0;

    while positions.len() < count as usize && attempts < 100 {
        if let Some(pos) = find_valid_position(grid, x_range.clone(), rng)
            && !positions.iter().any(|p| (*p - pos).length_squared() < 25)
        {
            positions.push(pos);
        }
        attempts += 1;
    }

    positions
}

const INVALID_SPAWN_TILES: [TileType; 2] = [TileType::Wall, TileType::DeadZone];

fn is_position_valid(grid: &TileGrid, x: i32, y: i32) -> bool {
    !INVALID_SPAWN_TILES.contains(&grid.get(IVec2::new(x, y)).unwrap_or(TileType::DeadZone))
}
