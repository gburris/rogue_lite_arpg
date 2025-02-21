use bevy_ecs_tilemap::map::TilemapSize;

use super::map_data::{MapDataBuilder, Prefab};
use crate::map::components::{MapLayout, TileType};

pub fn generate_hub_layout() -> MapLayout {
    let size = TilemapSize { x: 100, y: 100 };

    let map_data = MapDataBuilder::new(size)
        .with_floor(TileType::Grass)
        .with_exterior_walls()
        .with_prefab(Prefab::NPCHub)
        .build();
    return MapLayout::from(map_data);
}
