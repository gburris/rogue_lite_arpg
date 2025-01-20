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

#[derive(Debug)]
struct WallSection {
    start: (u32, u32),
    end: (u32, u32),
    is_horizontal: bool,
}

impl WallSection {
    fn new(start: (u32, u32), is_horizontal: bool) -> Self {
        WallSection {
            start,
            end: start,
            is_horizontal,
        }
    }

    fn extend(&mut self, pos: (u32, u32)) {
        self.end = pos;
    }

    fn length(&self) -> u32 {
        if self.is_horizontal {
            self.end.0 - self.start.0 + 1
        } else {
            self.end.1 - self.start.1 + 1
        }
    }
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

fn find_wall_sections(map_layout: &[Vec<TileType>], map_size: TilemapSize) -> Vec<WallSection> {
    let mut visited = vec![vec![false; map_size.y as usize]; map_size.x as usize];
    let mut wall_sections = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            if visited[x as usize][y as usize]
                || map_layout[x as usize][y as usize] != TileType::Wall
            {
                continue;
            }

            // Check horizontal wall
            if x + 1 < map_size.x && map_layout[(x + 1) as usize][y as usize] == TileType::Wall {
                let mut section = WallSection::new((x, y), true);
                let mut current_x = x + 1;

                visited[x as usize][y as usize] = true;

                while current_x < map_size.x
                    && map_layout[current_x as usize][y as usize] == TileType::Wall
                {
                    visited[current_x as usize][y as usize] = true;
                    section.extend((current_x, y));
                    current_x += 1;
                }

                wall_sections.push(section);
            }
            // Check vertical wall
            else if y + 1 < map_size.y
                && map_layout[x as usize][(y + 1) as usize] == TileType::Wall
            {
                let mut section = WallSection::new((x, y), false);
                let mut current_y = y + 1;

                visited[x as usize][y as usize] = true;

                while current_y < map_size.y
                    && map_layout[x as usize][current_y as usize] == TileType::Wall
                {
                    visited[x as usize][current_y as usize] = true;
                    section.extend((x, current_y));
                    current_y += 1;
                }

                wall_sections.push(section);
            }
            // Single wall tile
            else {
                visited[x as usize][y as usize] = true;
                wall_sections.push(WallSection::new((x, y), true));
            }
        }
    }

    wall_sections
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
    let wall_sections = find_wall_sections(&map_layout, map_size);

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
