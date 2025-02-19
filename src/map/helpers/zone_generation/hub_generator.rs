use bevy_ecs_tilemap::map::TilemapSize;

use super::map_data::{MapDataBuilder, MarkerPlacement};
use crate::map::components::{MapLayout, MapMarkers};

pub fn generate_hub_layout() -> MapLayout {
    let size = TilemapSize { x: 100, y: 100 };
    let hub_size = TilemapSize { x: 25, y: 25 };

    let map_data = MapDataBuilder::new(size)
        .with_grass_floor()
        .with_exterior_walls()
        .with_hub(hub_size)
        .with_marker_placement(MarkerPlacement::Hub)
        .build();

    MapLayout {
        size: map_data.size,
        tiles: map_data.tiles,
        markers: MapMarkers {
            markers: map_data.markers,
        },
        environmental_colliders: map_data.colliders,
    }
}
