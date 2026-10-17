use bevy::{log::warn, math::IVec2};
use bevy_ecs_tilemap::map::TilemapSize;
use rand::RngExt;

use crate::world::map::{
    blueprint::MapBlueprint,
    structures::{TEMPLE, apply_structure},
    tiles::{TileGrid, TileRect},
};

pub fn build_temple(blueprint: &mut MapBlueprint) {
    if let Some(bounds) = find_temple_position(&blueprint.tiles) {
        let _ = apply_structure(blueprint, &TEMPLE, bounds.min);
        // The C glyph in the temple picture adds its chest placement.
    } else {
        warn!("No valid temple position was found");
    }
}

const TEMPLE_WIDTH: u32 = 7;
const TEMPLE_HEIGHT: u32 = 7;

fn find_temple_position(tiles: &TileGrid) -> Option<TileRect> {
    let mut rng = rand::rng();
    let max_attempts = 100;

    let temple_size = TilemapSize {
        x: TEMPLE_WIDTH,
        y: TEMPLE_HEIGHT,
    };

    let base_bounds = TileRect::centered_in(tiles.map_size(), temple_size.into())?;

    for _ in 0..max_attempts {
        let offset_x =
            rng.random_range(-(tiles.map_size().x as i32 / 4)..(tiles.map_size().x as i32 / 4));
        let offset_y =
            rng.random_range(-(tiles.map_size().y as i32 / 4)..(tiles.map_size().y as i32 / 4));

        let Some(bounds) = base_bounds.translated(IVec2::new(offset_x, offset_y)) else {
            continue;
        };

        if tiles
            .rect_iter(&bounds)
            .is_some_and(|mut iter| iter.all(|(_, tile_type)| tile_type.can_spawn()))
        {
            return Some(bounds);
        }
    }
    None
}
