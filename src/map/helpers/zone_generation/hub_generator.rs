use bevy_ecs_tilemap::map::TilemapSize;

use super::map_data::{MapDataBuilder, Prefab};
use crate::map::components::{MapLayout, MapMarkers, TileType};

pub fn generate_hub_layout() -> MapLayout {
    let size = TilemapSize { x: 100, y: 100 };

    let map_data = MapDataBuilder::new(size)
        .with_floor(TileType::Grass)
        .with_exterior_walls()
        .with_prefab(Prefab::NPCHub)
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
