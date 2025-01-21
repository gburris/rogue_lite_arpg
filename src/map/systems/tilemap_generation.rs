use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    helpers::labels::GameCollisionLayer,
    map::{
        components::{TileType, Wall},
        helpers::map_layout::{find_wall_sections, generate_map_layout},
        resources::{CurrentZoneLevel, MapBounds, TileSize},
    },
};

pub fn generate_tilemap(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    mapbounds: Res<MapBounds>,
    tilesize: Res<TileSize>,
    zone_level: Res<CurrentZoneLevel>,
) {
    let ground_texture_handle: Handle<Image> = sprites.tiles.clone();
    let wall_texture_handle: Handle<Image> = sprites.wall_tiles.clone();

    let map_size: TilemapSize = TilemapSize {
        x: ((mapbounds.max_x - mapbounds.min_x) / tilesize.x) as u32,
        y: ((mapbounds.max_y - mapbounds.min_y) / tilesize.y) as u32,
    };

    let map_layout = generate_map_layout(map_size);
    let wall_sections = find_wall_sections(&map_layout, map_size);

    let mut ground_storage = TileStorage::empty(map_size);
    let mut wall_storage = TileStorage::empty(map_size);

    let map_type = TilemapType::Square;
    let ground_tilemap_entity = commands.spawn_empty().id();
    let wall_tilemap_entity = commands.spawn_empty().id();
    let spawn_tilemap_entity = commands.spawn_empty().id();

    // Calculate tilemap offset for centered position
    let tile_size = TilemapTileSize {
        x: tilesize.x,
        y: tilesize.y,
    };
    let grid_size = tile_size.into();
    let offset = Vec2::new(
        -((map_size.x as f32 * tilesize.x) / 2.0),
        -((map_size.y as f32 * tilesize.y) / 2.0),
    );

    // Spawn tiles without colliders
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let color = zone_level.0 % 6;

            match map_layout[x as usize][y as usize] {
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
                TileType::SpawnTile => {
                    let spawn_tile = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(spawn_tilemap_entity),
                            texture_index: TileTextureIndex(0),
                            ..Default::default()
                        })
                        .id();
                    wall_storage.set(&tile_pos, spawn_tile);
                }
                TileType::ExitTile => todo!(),
            }
        }
    }

    // Spawn colliders for wall sections
    for section in wall_sections {
        let start_pos = Vec2::new(
            section.start.0 as f32 * tilesize.x,
            section.start.1 as f32 * tilesize.y,
        ) + offset;

        let length = section.length() as f32;
        let (width, height) = if section.is_horizontal {
            (length * tilesize.x, tilesize.y)
        } else {
            (tilesize.x, length * tilesize.y)
        };

        let collider_pos = if section.is_horizontal {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + (height / 2.0))
        } else {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + (height / 2.0))
        };

        commands.spawn((
            Wall,
            Collider::rectangle(width, height),
            RigidBody::Static,
            CollisionLayers::new(
                GameCollisionLayer::Wall,
                [
                    GameCollisionLayer::Player,
                    GameCollisionLayer::Npc,
                    GameCollisionLayer::Enemy,
                    GameCollisionLayer::Projectile,
                ],
            ),
            Transform::from_xyz(collider_pos.x, collider_pos.y, 1.0),
            GlobalTransform::default(),
        ));
    }

    // Spawn tilemaps
    commands
        .entity(ground_tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            size: map_size,
            storage: ground_storage,
            map_type,
            texture: TilemapTexture::Single(ground_texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        });

    commands.entity(wall_tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: wall_storage,
        map_type,
        texture: TilemapTexture::Single(wall_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
        ..Default::default()
    });
}
