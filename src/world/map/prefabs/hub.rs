use bevy::math::{IVec2, UVec2};

use crate::{
    prelude::NpcType,
    world::map::{
        blueprint::{MapBlueprint, Placement, PlacementKind},
        tiles::{TileGrid, TileRect, TileType},
    },
};

const HUB_SIZE: UVec2 = UVec2::new(25, 25);
const PLAYER_Y_TILE_OFFSET: i32 = 5;
const LEVEL_EXIT_Y_OFFSET: f32 = 22.0;
const NPC_OFFSET: f32 = 5.0;
const ENTRANCE_WIDTH: i32 = 5;

pub fn build_hub(blueprint: &mut MapBlueprint) {
    let hub_bounds = TileRect::centered_in(blueprint.map_size(), HUB_SIZE).unwrap();
    blueprint
        .tiles
        .fill_rect(&hub_bounds, TileType::Cobblestone);

    add_hub_walls(&mut blueprint.tiles, &hub_bounds);
    add_hub_entrance(&mut blueprint.tiles, &hub_bounds);

    blueprint.placements.extend([
        Placement {
            kind: PlacementKind::PlayerStart,
            tile: IVec2::new(
                hub_bounds.center_tile().x,
                hub_bounds.min.y + PLAYER_Y_TILE_OFFSET,
            ),
        },
        Placement {
            kind: PlacementKind::Exit,
            tile: IVec2::new(
                hub_bounds.center_tile().x,
                hub_bounds.min.y + LEVEL_EXIT_Y_OFFSET as i32,
            ),
        },
        Placement {
            kind: PlacementKind::Npc(NpcType::Shopkeeper),
            tile: IVec2::new(
                hub_bounds.center_tile().x + NPC_OFFSET as i32,
                hub_bounds.center_tile().y + NPC_OFFSET as i32,
            ),
        },
        Placement {
            kind: PlacementKind::Npc(NpcType::Helper),
            tile: IVec2::new(
                hub_bounds.center_tile().x - NPC_OFFSET as i32,
                hub_bounds.center_tile().y - NPC_OFFSET as i32,
            ),
        },
        Placement {
            kind: PlacementKind::Npc(NpcType::StatTrainer),
            tile: IVec2::new(
                hub_bounds.center_tile().x + NPC_OFFSET as i32,
                hub_bounds.center_tile().y - NPC_OFFSET as i32,
            ),
        },
    ]);
}

fn add_hub_walls(tiles: &mut TileGrid, tile_rect: &TileRect) {
    let min_x = tile_rect.min.x;
    let max_x = tile_rect.min.x + tile_rect.size.x as i32;
    let min_y = tile_rect.min.y;
    let max_y = tile_rect.min.y + tile_rect.size.y as i32;
    let wall_thickness = 2;

    // Add horizontal walls (top and bottom)
    for wall_layer in 0..wall_thickness {
        // Top wall
        if min_y + wall_layer < max_y {
            let start_x = min_x;
            let y = min_y + wall_layer;

            for x in start_x..max_x {
                tiles.set(IVec2::new(x, y), TileType::Wall);
            }
        }

        // Bottom wall
        if max_y - wall_layer > min_y {
            let start_x = min_x;
            let y = max_y - wall_layer - 1;

            for x in start_x..max_x {
                tiles.set(IVec2::new(x, y), TileType::Wall);
            }
        }
    }

    // Add vertical walls (left and right)
    for wall_layer in 0..wall_thickness {
        // Left wall
        if min_x + wall_layer < max_x {
            let x = min_x + wall_layer;

            for y in min_y..max_y {
                tiles.set(IVec2::new(x, y), TileType::Wall);
            }
        }

        // Right wall
        if max_x - wall_layer > min_x {
            let x = max_x - wall_layer - 1;

            for y in min_y..max_y {
                tiles.set(IVec2::new(x, y), TileType::Wall);
            }
        }
    }
}

fn add_hub_entrance(tiles: &mut TileGrid, tile_rect: &TileRect) {
    let entrance_x_start =
        i32::midpoint(tile_rect.min.x, tile_rect.min.x + tile_rect.size.x as i32)
            - ENTRANCE_WIDTH / 2;

    // Create the entrance path
    let y_range_start = tile_rect.min.y - 5;

    let entrance_bounds = TileRect {
        min: IVec2::new(entrance_x_start, y_range_start),
        size: UVec2::new(ENTRANCE_WIDTH as u32, 10),
    };

    tiles.fill_rect(&entrance_bounds, TileType::Wood);
}
