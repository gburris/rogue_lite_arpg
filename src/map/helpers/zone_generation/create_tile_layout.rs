use bevy_ecs_tilemap::map::TilemapSize;

use super::{
    dead_zone::add_dead_zones, hub::build_hub, map_data::MapData, utils::calculate_center_rect,
    walls::add_exterior_walls,
};

pub fn create_hub(map_size: TilemapSize, hub_size: TilemapSize) -> MapData {
    let mut map_data = MapData::new_with_grass(map_size);

    // Add exterior walls with colliders
    add_exterior_walls(&mut map_data, map_size);

    // Calculate hub bounds
    let hub_bounds = calculate_center_rect(map_size, hub_size);

    build_hub(&mut map_data, &hub_bounds);

    map_data
}

pub fn create_map_with_exterior_walls_and_dead_zones(
    map_size: TilemapSize,
    should_make_zones: bool,
) -> MapData {
    let mut map_data = MapData::new(map_size);
    // Add exterior walls and their colliders
    add_exterior_walls(&mut map_data, map_size);

    // Add dead zones if requested
    if should_make_zones {
        add_dead_zones(&mut map_data, map_size);
    }

    map_data
}
