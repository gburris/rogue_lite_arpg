use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    helpers::labels::GameCollisionLayer,
    map::resources::{CurrentZoneLevel, MapBounds, TileSize},
};

#[derive(Component)]
pub struct Wall;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Ground,
    Wall,
}

// Rest of generate_map_layout remains the same...
// [Previous generate_map_layout implementation]

pub fn generate_map_layout(map_size: TilemapSize) -> Vec<Vec<TileType>> {
    let mut rng = rand::thread_rng();
    let mut map = vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize];

    // Reduce number of walls to about 2% of map size
    let num_walls = (map_size.x as f32 * map_size.y as f32 * 0.002) as i32;

    for _ in 0..num_walls {
        // Increased maximum wall length for more interesting structures
        let wall_length = rng.gen_range(10..25); // Random length between 10-25 tiles

        // Add some padding from the edges
        let start_x = rng.gen_range(5..(map_size.x as i32 - wall_length));
        let start_y = rng.gen_range(5..(map_size.y as i32 - wall_length));

        // Randomly choose horizontal or vertical wall
        let is_horizontal = rng.gen_bool(0.5);

        // Only place wall if area is clear (to prevent overcrowding)
        let can_place = |x: i32, y: i32| -> bool {
            let padding = 3; // Space between walls
            for dx in -padding..=padding {
                for dy in -padding..=padding {
                    let check_x = x + dx;
                    let check_y = y + dy;
                    if check_x >= 0
                        && check_x < map_size.x as i32
                        && check_y >= 0
                        && check_y < map_size.y as i32
                        && map[check_x as usize][check_y as usize] == TileType::Wall
                    {
                        return false;
                    }
                }
            }
            true
        };

        // Only place wall if entire length is clear
        let mut can_place_wall = true;
        for i in 0..wall_length {
            let (check_x, check_y) = if is_horizontal {
                (start_x + i, start_y)
            } else {
                (start_x, start_y + i)
            };

            if !can_place(check_x, check_y) {
                can_place_wall = false;
                break;
            }
        }

        // Place wall if area is clear
        if can_place_wall {
            for i in 0..wall_length {
                if is_horizontal {
                    map[(start_x + i) as usize][start_y as usize] = TileType::Wall;
                } else {
                    map[start_x as usize][(start_y + i) as usize] = TileType::Wall;
                }
            }
        }
    }

    map
}

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

    let mut ground_storage = TileStorage::empty(map_size);
    let mut wall_storage = TileStorage::empty(map_size);

    let map_type = TilemapType::Square;

    let ground_tilemap_entity = commands.spawn_empty().id();
    let wall_tilemap_entity = commands.spawn_empty().id();

    // Calculate tilemap offset for centered position
    let tile_size = TilemapTileSize {
        x: tilesize.x,
        y: tilesize.y,
    };
    let grid_size = tile_size.into();

    // Get the centered transform that the tilemap will use
    let tilemap_transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);
    let offset = Vec2::new(
        -((map_size.x as f32 * tilesize.x) / 2.0),
        -((map_size.y as f32 * tilesize.y) / 2.0),
    );

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let color = zone_level.0 % 6;

            // Calculate world position for the tile, accounting for the centered tilemap
            let world_pos = Vec2::new(
                x as f32 * tilesize.x + tilesize.x / 2.0,
                y as f32 * tilesize.y + tilesize.y / 2.0,
            ) + offset;

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
                        .spawn((
                            TileBundle {
                                position: tile_pos,
                                tilemap_id: TilemapId(wall_tilemap_entity),
                                texture_index: TileTextureIndex(0),
                                ..Default::default()
                            },
                            Wall,
                            Collider::rectangle(tilesize.x, tilesize.y),
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
                            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
                            GlobalTransform::default(),
                        ))
                        .id();
                    wall_storage.set(&tile_pos, wall_entity);
                }
            }
        }
    }

    commands
        .entity(ground_tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            size: map_size,
            storage: ground_storage,
            map_type,
            texture: TilemapTexture::Single(ground_texture_handle),
            tile_size,
            transform: tilemap_transform,
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
