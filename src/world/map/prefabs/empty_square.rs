use bevy::{
    log::warn,
    math::{IVec2, UVec2},
};
use rand::RngExt;

use crate::world::map::{
    blueprint::MapBlueprint,
    tiles::{TileGrid, TileRect, TileType},
};

pub fn build_empty_square(blueprint: &mut MapBlueprint) {
    if let Some(bounds) = find_dead_zone_position(&blueprint.tiles) {
        add_dead_zone_structure(blueprint, &bounds);
    } else {
        warn!("No valid dead zone position was found");
    }
}

fn find_dead_zone_position(map: &TileGrid) -> Option<TileRect> {
    let mut rng = rand::rng();
    let max_attempts = 50;
    let size = rng.random_range(3..=10) as i32;
    let min_distance = 4;

    for _ in 0..max_attempts {
        let map_size = map.map_size();
        let max_x = (map_size.x as i32 - size - min_distance).max(0);
        let max_y = (map_size.y as i32 - size - min_distance).max(0);
        let min_x = min_distance;
        let min_y = min_distance;

        if max_x <= min_x || max_y <= min_y {
            continue;
        }

        let start_x = rng.random_range(min_x..max_x);
        let start_y = rng.random_range(min_y..max_y);

        let bounds = TileRect {
            min: IVec2::new(start_x, start_y),
            size: UVec2::new(size as u32, size as u32),
        };

        if can_place_dead_zone(map, &bounds) {
            return Some(bounds);
        }
    }
    None
}

fn can_place_dead_zone(tiles: &TileGrid, bounds: &TileRect) -> bool {
    let Some(padded_bounds) = bounds.padded(2) else {
        return false;
    };

    if !tiles.contains_rect(&padded_bounds) {
        return false;
    }

    tiles
        .rect_iter(&padded_bounds)
        .is_some_and(|mut iter| iter.all(|(_, tile_type)| tile_type.can_spawn()))
}

fn add_dead_zone_structure(blueprint: &mut MapBlueprint, bounds: &TileRect) {
    let start_x = bounds.min.x as i32;
    let start_y = bounds.min.y as i32;
    let size = (bounds.size.x as i32).max(3);

    blueprint.tiles.fill_rect(bounds, TileType::DeadZone);

    // Add walls and colliders around perimeter
    if start_x > 0 && start_y > 0 {
        // Top wall
        for x in (start_x - 1)..=(start_x + size) {
            blueprint
                .tiles
                .set(IVec2::new(x, start_y - 1), TileType::Wall);
        }

        // Bottom wall
        for x in (start_x - 1)..=(start_x + size) {
            blueprint
                .tiles
                .set(IVec2::new(x, start_y + size), TileType::Wall);
        }

        // Left wall
        for y in (start_y - 1)..=(start_y + size) {
            blueprint
                .tiles
                .set(IVec2::new(start_x - 1, y), TileType::Wall);
        }

        // Right wall
        for y in (start_y - 1)..=(start_y + size) {
            blueprint
                .tiles
                .set(IVec2::new(start_x + size, y), TileType::Wall);
        }
    }
}
