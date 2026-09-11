use bevy::{platform::collections::HashSet, prelude::*};
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

    let mut occupied: HashSet<IVec2> = blueprint.placements.iter().map(|p| p.tile).collect();

    // Generate player spawn and exits
    let (player_start, exits) =
        generate_entrance_exit_positions(blueprint.map_size(), map_recipe.num_exits, rng);

    for exit in exits {
        if !blueprint
            .tiles
            .get(exit.tile)
            .is_some_and(|t: TileType| t.can_spawn())
        {
            return Err(format!("exit tile {:?} is not spawnable", exit.tile).into());
        }

        if !occupied.insert(exit.tile) {
            return Err(format!("exit tile {:?} is already occupied", exit.tile).into());
        }

        if !blueprint.tiles.contains(exit.doorway) {
            return Err(format!("exit doorway {:?} is out of bounds", exit.doorway).into());
        }

        blueprint.placements.push(Placement {
            tile: exit.tile,
            kind: PlacementKind::Exit,
        });
    }

    // Player start
    let has_existing_player_start = blueprint
        .placements
        .iter()
        .any(|p| matches!(p.kind, PlacementKind::PlayerStart));
    if !has_existing_player_start {
        if !blueprint
            .tiles
            .get(player_start)
            .is_some_and(|t: TileType| t.can_spawn())
        {
            return Err(format!("player start {:?} is not spawnable", player_start).into());
        }

        if !occupied.insert(player_start) {
            return Err(format!("player start {:?} is already occupied", player_start).into());
        }

        blueprint.placements.push(Placement {
            tile: player_start,
            kind: PlacementKind::PlayerStart,
        });
    }

    // Determine number of enemies and chests from the recipe ranges
    let num_enemies = rng
        .random_range(map_recipe.number_of_enemies_range.0..=map_recipe.number_of_enemies_range.1)
        as u32;
    let num_chests = rng.random_range(map_recipe.chest_range.0..=map_recipe.chest_range.1) as u32;

    // Generate enemy positions (avoid walls)
    let enemy_positions =
        find_multiple_positions(&blueprint.tiles, 2..8, num_enemies, rng, &mut occupied);
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
    let chest_positions =
        find_multiple_positions(&blueprint.tiles, 1..9, num_chests, rng, &mut occupied);
    for pos in chest_positions {
        blueprint.placements.push(Placement {
            tile: pos,
            kind: PlacementKind::Chest,
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

struct ExitPlacement {
    tile: IVec2,
    doorway: IVec2,
}

fn generate_entrance_exit_positions(
    map_size: UVec2,
    num_exits: u32,
    rng: &mut ChaCha8Rng,
) -> (IVec2, Vec<ExitPlacement>) {
    let orientation = determine_map_orientation(map_size);

    let player_start = match orientation {
        MapOrientation::Horizontal | MapOrientation::Square => {
            // For horizontal and square maps: left to right
            let player_x = 1; // One tile in from left wall
            let player_y = rng.random_range(1..map_size.y - 1) as i32;
            IVec2::new(player_x, player_y)
        }
        MapOrientation::Vertical => {
            // For vertical maps: top to bottom
            let player_y = map_size.y as i32 - 2; // One tile down from top wall
            let player_x = rng.random_range(1..map_size.x - 1) as i32;
            IVec2::new(player_x, player_y)
        }
    };

    let mut exits = match orientation {
        MapOrientation::Horizontal | MapOrientation::Square => {
            // Two exits on the right side
            let exit_y1 = rng.random_range(1..map_size.y / 2) as i32;
            let exit_y2 = rng.random_range(map_size.y / 2..map_size.y - 1) as i32;

            let interior_x = map_size.x as i32 - 2;
            let boundary_x = map_size.x as i32 - 1;

            vec![
                ExitPlacement {
                    tile: IVec2::new(interior_x, exit_y1),
                    doorway: IVec2::new(boundary_x, exit_y1),
                },
                ExitPlacement {
                    tile: IVec2::new(interior_x, exit_y2),
                    doorway: IVec2::new(boundary_x, exit_y2),
                },
            ]
        }
        MapOrientation::Vertical => {
            // Two exits at the bottom
            let exit_x1 = rng.random_range(1..map_size.x / 2) as i32;
            let exit_x2 = rng.random_range(map_size.x / 2..map_size.x - 1) as i32;

            vec![
                ExitPlacement {
                    tile: IVec2::new(exit_x1, 1),
                    doorway: IVec2::new(exit_x1, 0),
                },
                ExitPlacement {
                    tile: IVec2::new(exit_x2, 1),
                    doorway: IVec2::new(exit_x2, 0),
                },
            ]
        }
    };
    exits.truncate(num_exits as usize);

    (player_start, exits)
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

fn find_multiple_positions(
    grid: &TileGrid,
    x_range: std::ops::Range<u32>,
    count: u32,
    rng: &mut ChaCha8Rng,
    occupied: &mut HashSet<IVec2>,
) -> Vec<IVec2> {
    let mut positions: Vec<IVec2> = Vec::new();
    let mut attempts = 0;

    while positions.len() < count as usize && attempts < 100 {
        if let Some(pos) = find_valid_position(grid, x_range.clone(), rng, occupied)
            && positions
                .iter()
                .all(|existing| (*existing - pos).length_squared() >= 25)
        {
            positions.push(pos);
            occupied.insert(pos);
        }
        attempts += 1;
    }

    positions
}

fn find_valid_position(
    tiles: &TileGrid,
    x_range: std::ops::Range<u32>,
    rng: &mut ChaCha8Rng,
    occupied: &mut HashSet<IVec2>,
) -> Option<IVec2> {
    let x_start = tiles.map_size().x * x_range.start / 10;
    let x_end = tiles.map_size().x * x_range.end / 10;
    for _ in 0..100 {
        let random_tile = IVec2::new(
            rng.random_range(x_start..x_end) as i32,
            rng.random_range(1..tiles.map_size().y - 1) as i32,
        );

        if !occupied.contains(&random_tile) && tiles.get(random_tile).is_some_and(|t| t.can_spawn())
        {
            return Some(random_tile);
        }
    }
    None
}
