use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    labels::layer::ZLayer,
    map::{
        components::{MapLayout, TileType},
        resources::{CurrentZoneLevel, TileSize},
        WorldSpaceConfig,
    },
};

pub fn render_instance_tilemap(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    zone_level: Res<CurrentZoneLevel>,
) {
    let water_texture_handle: Handle<Image> = sprites.water_tiles.clone();
    let ground_texture_handle: Handle<Image> = sprites.tiles.clone();
    let wall_texture_handle: Handle<Image> = sprites.wall_tiles.clone();
    let map_size = world_config.map_size;
    let tile_size = world_config.tile_size;

    // Set up storage for each tile type
    let mut ground_storage = TileStorage::empty(map_size);
    let mut wall_storage = TileStorage::empty(map_size);
    let mut water_storage = TileStorage::empty(map_size);

    // Create empty entities for each tilemap layer
    let map_type = TilemapType::Square;
    let ground_tilemap_entity = commands.spawn_empty().id();
    let wall_tilemap_entity = commands.spawn_empty().id();
    let water_tilemap_entity = commands.spawn_empty().id();

    let grid_size = tile_size.into();

    // Spawn tiles
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let color = zone_level.0 % 6;

            match map_layout.tiles[x as usize][y as usize] {
                TileType::Ground => {
                    let ground_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(ground_tilemap_entity),
                            texture_index: TileTextureIndex(color),
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
                            texture_index: TileTextureIndex(0),
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
                            texture_index: TileTextureIndex(0),
                            ..Default::default()
                        })
                        .id();
                    water_storage.set(&tile_pos, water_entity);
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
}
