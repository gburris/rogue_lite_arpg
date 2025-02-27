use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;
use std::{collections::HashMap, sync::OnceLock};

use crate::{
    configuration::assets::SpriteAssets,
    labels::layer::ZLayer,
    map::components::{MapLayout, TileType, WorldSpaceConfig},
};

#[derive(Clone, Copy)]
pub enum TileIndexType {
    Random(u32), // Maximum random value
    Fixed(u32),  // Fixed index
}

fn tile_configurations() -> &'static HashMap<TileType, TileIndexType> {
    static CONFIGS: OnceLock<HashMap<TileType, TileIndexType>> = OnceLock::new();
    CONFIGS.get_or_init(|| {
        let mut m = HashMap::new();

        m.insert(TileType::Ground, TileIndexType::Random(10));
        m.insert(TileType::Grass, TileIndexType::Random(10));
        m.insert(TileType::Wall, TileIndexType::Fixed(0));
        m.insert(TileType::Water, TileIndexType::Fixed(0));
        m.insert(TileType::Wood, TileIndexType::Random(10));
        m.insert(TileType::Cobblestone, TileIndexType::Random(10));

        m
    })
}

pub fn spawn_zone_tilemap(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    sprites: Res<SpriteAssets>,
) {
    let map_size = map_layout.size;
    let tile_size = world_config.tile_size;
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::Square;

    // Create storage and entities for each tile type
    let mut storages: HashMap<TileType, (Entity, TileStorage)> = HashMap::new();

    // Get texture handles based on tile type
    let texture_handles: HashMap<TileType, Handle<Image>> = HashMap::from([
        (TileType::Ground, sprites.ground_tiles.clone()),
        (TileType::Grass, sprites.grass_tiles.clone()),
        (TileType::Wall, sprites.wall_tiles.clone()),
        (TileType::Water, sprites.water_tiles.clone()),
        (TileType::Wood, sprites.wood_tiles.clone()),
        (TileType::Cobblestone, sprites.cobblestone_tiles.clone()),
    ]);

    // Initialize storage for each tile type
    for tile_type in tile_configurations().keys() {
        let tilemap_entity = commands.spawn_empty().id();
        let storage = TileStorage::empty(map_size);
        storages.insert(*tile_type, (tilemap_entity, storage));
    }

    // Spawn tiles
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_type = map_layout.tiles[x as usize][y as usize];

            if let Some((tilemap_entity, storage)) = storages.get_mut(&tile_type) {
                if let Some(index_type) = tile_configurations().get(&tile_type) {
                    let texture_index = match index_type {
                        TileIndexType::Random(max) => rand::thread_rng().gen_range(0..*max),
                        TileIndexType::Fixed(index) => *index,
                    };

                    let tile_entity = commands
                        .spawn((
                            Name::new("Tile"),
                            TileBundle {
                                position: tile_pos,
                                tilemap_id: TilemapId(*tilemap_entity),
                                texture_index: TileTextureIndex(texture_index),
                                ..Default::default()
                            },
                        ))
                        .id();
                    storage.set(&tile_pos, tile_entity);
                }
            }
        }
    }

    // Insert tilemaps
    for (tile_type, (entity, storage)) in storages {
        if let Some(texture_handle) = texture_handles.get(&tile_type) {
            commands.entity(entity).insert(TilemapBundle {
                grid_size,
                size: map_size,
                storage,
                map_type,
                texture: TilemapTexture::Single(texture_handle.clone()),
                tile_size,
                transform: get_tilemap_center_transform(
                    &map_size,
                    &grid_size,
                    &map_type,
                    ZLayer::Ground.z(),
                ),
                ..Default::default()
            });
        }
    }
}
