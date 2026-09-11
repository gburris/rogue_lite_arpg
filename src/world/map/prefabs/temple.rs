use bevy::{log::warn, math::IVec2};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::RngExt;

use crate::world::map::{
    TileType,
    blueprint::{MapBlueprint, Placement, PlacementKind, TileGrid, TileRect},
};

pub fn build_temple(blueprint: &mut MapBlueprint) {
    if let Some(bounds) = find_temple_position(&blueprint.tiles) {
        add_temple_structure(blueprint, &bounds);

        blueprint.placements.extend([Placement {
            kind: PlacementKind::Chest,
            tile: IVec2::new(
                bounds.min.x + (TEMPLE_WIDTH / 2) as i32,
                bounds.min.y + (TEMPLE_HEIGHT / 2) as i32,
            ),
        }]);
    } else {
        warn!("No valid temple position was found");
    }
}

const TEMPLE_WIDTH: u32 = 7;
const TEMPLE_HEIGHT: u32 = 7;
const ENTRANCE_WIDTH: u32 = 3;

fn find_temple_position(grid: &TileGrid) -> Option<TileRect> {
    let mut rng = rand::rng();
    let max_attempts = 100;

    let temple_size = TilemapSize {
        x: TEMPLE_WIDTH,
        y: TEMPLE_HEIGHT,
    };

    let base_bounds = TileRect::centered_in(grid.map_size(), temple_size.into())?;

    for _ in 0..max_attempts {
        let offset_x =
            rng.random_range(-(grid.map_size().x as i32 / 4)..(grid.map_size().x as i32 / 4));
        let offset_y =
            rng.random_range(-(grid.map_size().y as i32 / 4)..(grid.map_size().y as i32 / 4));

        let Some(bounds) = base_bounds.translated(IVec2::new(offset_x, offset_y)) else {
            continue;
        };

        if grid
            .rect_iter(&bounds)
            .is_some_and(|mut iter| iter.all(|(_, tile_type)| tile_type.can_spawn()))
        {
            return Some(bounds);
        }
    }
    None
}

fn add_temple_structure(blueprint: &mut MapBlueprint, bounds: &TileRect) {
    let start_x = bounds.min.x as u32;
    let start_y = bounds.min.y as u32;

    blueprint.tiles.fill_rect(bounds, TileType::Cobblestone);

    let entrance_start = start_x + (TEMPLE_WIDTH - ENTRANCE_WIDTH) / 2;

    let grid_size = blueprint.map_size();
    for x in start_x..start_x + TEMPLE_WIDTH {
        for y in start_y..start_y + TEMPLE_HEIGHT {
            // Bounds check using map_size()
            if x < grid_size.x && y < grid_size.y {
                let is_wall = x == start_x
                    || x == start_x + TEMPLE_WIDTH - 1
                    || y == start_y
                    || (y == start_y + TEMPLE_HEIGHT - 1
                        && (x < entrance_start || x >= entrance_start + ENTRANCE_WIDTH));
                if is_wall {
                    let _ = blueprint
                        .tiles
                        .set(IVec2::new(x as i32, y as i32), TileType::Wall);
                }
            }
        }
    }
}
