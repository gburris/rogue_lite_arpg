use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;

use crate::{
    configuration::assets::SpriteAssets,
    labels::layer::ZLayer,
    map::{MapLayout, TileType, WorldSpaceConfig},
};

pub fn render_hub_tiles(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
) {
    let mut rng = rand::thread_rng();
    let water_texture_handle: Handle<Image> = sprites.water_tiles.clone();
    let ground_texture_handle: Handle<Image> = sprites.grass_tiles.clone();
    let wood_texture_handle: Handle<Image> = sprites.wood_tiles.clone();
    let wall_texture_handle: Handle<Image> = sprites.wall_tiles.clone();
    let cobblestone_texture_handle: Handle<Image> = sprites.cobblestone_tiles.clone();
    let map_size = world_config.map_size;
    let tile_size = world_config.tile_size;

    let grid_size: TilemapGridSize = tile_size.into();

    // Set up storage for each tile type
    let mut ground_storage = TileStorage::empty(map_size);
    let mut wood_storage = TileStorage::empty(map_size);
    let mut wall_storage = TileStorage::empty(map_size);
    let mut water_storage = TileStorage::empty(map_size);
    let mut cobblestone_storage = TileStorage::empty(map_size);

    let ground_tilemap_entity = commands.spawn_empty().id();
    let wood_tilemap_entity = commands.spawn_empty().id();
    let wall_tilemap_entity = commands.spawn_empty().id();
    let water_tilemap_entity = commands.spawn_empty().id();
    let cobblestone_tilemap_entity = commands.spawn_empty().id();

    let map_size = world_config.map_size;
    let tile_size = world_config.tile_size;

    let map_type = TilemapType::Square;

    // Spawn tiles
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let random_tile_index = rng.gen_range(0..10);

            match map_layout.tiles[x as usize][y as usize] {
                TileType::Ground => {
                    let ground_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(ground_tilemap_entity),
                            texture_index: TileTextureIndex(random_tile_index),
                            ..Default::default()
                        })
                        .id();
                    ground_storage.set(&tile_pos, ground_entity);
                }
                TileType::Wall => {
                    let wall_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(wall_tilemap_entity),
                            texture_index: TileTextureIndex(random_tile_index),
                            ..Default::default()
                        })
                        .id();
                    wall_storage.set(&tile_pos, wall_entity);
                }
                TileType::Water => {
                    let water_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(water_tilemap_entity),
                            texture_index: TileTextureIndex(random_tile_index),
                            ..Default::default()
                        })
                        .id();
                    water_storage.set(&tile_pos, water_entity);
                }
                TileType::Wood => {
                    let wood_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(wood_tilemap_entity),
                            texture_index: TileTextureIndex(random_tile_index),
                            ..Default::default()
                        })
                        .id();
                    wood_storage.set(&tile_pos, wood_entity);
                }
                TileType::Cobblestone => {
                    let cobblestone_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(cobblestone_tilemap_entity),
                            texture_index: TileTextureIndex(random_tile_index),
                            ..Default::default()
                        })
                        .id();
                    cobblestone_storage.set(&tile_pos, cobblestone_entity);
                }
            }
        }
    }

    // Insert the ground tilemap
    commands
        .entity(ground_tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            size: map_size,
            storage: ground_storage,
            map_type,
            texture: TilemapTexture::Single(ground_texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(
                &map_size,
                &grid_size,
                &map_type,
                ZLayer::Ground.z(),
            ),
            ..Default::default()
        });

    // Insert the wall tilemap
    commands.entity(wall_tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: wall_storage,
        map_type,
        texture: TilemapTexture::Single(wall_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(
            &map_size,
            &grid_size,
            &map_type,
            ZLayer::Ground.z(),
        ),
        ..Default::default()
    });

    // Insert the water tilemap
    commands.entity(water_tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: water_storage,
        map_type,
        texture: TilemapTexture::Single(water_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(
            &map_size,
            &grid_size,
            &map_type,
            ZLayer::Ground.z(),
        ),
        ..Default::default()
    });
    // Insert the wood tilemap
    commands.entity(wood_tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: wood_storage,
        map_type,
        texture: TilemapTexture::Single(wood_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(
            &map_size,
            &grid_size,
            &map_type,
            ZLayer::Ground.z(),
        ),
        ..Default::default()
    });
    commands
        .entity(cobblestone_tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            size: map_size,
            storage: cobblestone_storage,
            map_type,
            texture: TilemapTexture::Single(cobblestone_texture_handle),
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
