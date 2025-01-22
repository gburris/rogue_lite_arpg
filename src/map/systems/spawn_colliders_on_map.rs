use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    helpers::labels::GameCollisionLayer,
    map::{
        components::{MapLayout, TileType, Wall, Water},
        resources::TileSize,
        WallSection,
    },
};

pub fn process_map_collisions_zones(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
    tile_size: Res<TileSize>,
) {
    let map_size = TilemapSize {
        x: map_layout.tiles.len() as u32,
        y: map_layout.tiles[0].len() as u32,
    };

    // Calculate center offset based on tilemap centering logic
    let grid_size = TilemapGridSize::new(tile_size.x, tile_size.x);
    let map_type = TilemapType::Square;

    let low = TilePos::new(0, 0).center_in_world(&grid_size, &map_type);
    let high = TilePos::new(map_size.x, map_size.y).center_in_world(&grid_size, &map_type);
    let diff = high - low;
    let offset = Vec2::new(-diff.x / 2.0, -diff.y / 2.0);

    // Process wall colliders
    let wall_sections = find_wall_sections(&map_layout.tiles, map_size);
    spawn_wall_colliders(&mut commands, &wall_sections, &tile_size, offset);

    // Process water ponds
    let water_ponds = find_water_ponds(&map_layout.tiles, map_size);
    spawn_water_pond_colliders(&mut commands, &water_ponds, &tile_size, offset);
}

fn find_wall_sections(tiles: &[Vec<TileType>], map_size: TilemapSize) -> Vec<WallSection> {
    let mut visited = vec![vec![false; map_size.y as usize]; map_size.x as usize];
    let mut wall_sections = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            if !visited[x as usize][y as usize] && tiles[x as usize][y as usize] == TileType::Wall {
                if let Some(section) = extract_wall_section(tiles, map_size, x, y, &mut visited) {
                    wall_sections.push(section);
                }
            }
        }
    }

    wall_sections
}

fn extract_wall_section(
    tiles: &[Vec<TileType>],
    map_size: TilemapSize,
    x: u32,
    y: u32,
    visited: &mut Vec<Vec<bool>>,
) -> Option<WallSection> {
    visited[x as usize][y as usize] = true;

    // Try horizontal first
    if x + 1 < map_size.x && tiles[(x + 1) as usize][y as usize] == TileType::Wall {
        let mut section = WallSection::new((x, y), true);
        let mut current_x = x + 1;

        while current_x < map_size.x && tiles[current_x as usize][y as usize] == TileType::Wall {
            visited[current_x as usize][y as usize] = true;
            section.extend((current_x, y));
            current_x += 1;
        }

        Some(section)
    }
    // Then try vertical
    else if y + 1 < map_size.y && tiles[x as usize][(y + 1) as usize] == TileType::Wall {
        let mut section = WallSection::new((x, y), false);
        let mut current_y = y + 1;

        while current_y < map_size.y && tiles[x as usize][current_y as usize] == TileType::Wall {
            visited[x as usize][current_y as usize] = true;
            section.extend((x, current_y));
            current_y += 1;
        }

        Some(section)
    }
    // Single wall tile
    else {
        Some(WallSection::new((x, y), true))
    }
}

fn spawn_wall_colliders(
    commands: &mut Commands,
    wall_sections: &[WallSection],
    tile_size: &TileSize,
    offset: Vec2,
) {
    for section in wall_sections {
        let start_pos = Vec2::new(
            section.start.0 as f32 * tile_size.x,
            section.start.1 as f32 * tile_size.y,
        );

        let length = section.length() as f32;
        let (width, height) = if section.is_horizontal {
            (length * tile_size.x, tile_size.y)
        } else {
            (tile_size.x, length * tile_size.y)
        };

        let collider_pos = if section.is_horizontal {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + (height / 2.0))
        } else {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + (height / 2.0))
        };

        // Apply the offset to center the collider
        let adjusted_pos = collider_pos + offset;

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
            Transform::from_xyz(adjusted_pos.x, adjusted_pos.y, 1.0),
            GlobalTransform::default(),
        ));
    }
}

#[derive(Debug)]
struct WaterPond {
    center: Vec2,
    radius: f32,
}

fn find_water_ponds(tiles: &[Vec<TileType>], map_size: TilemapSize) -> Vec<WaterPond> {
    let mut visited = vec![vec![false; map_size.y as usize]; map_size.x as usize];
    let mut ponds = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            if !visited[x as usize][y as usize] && tiles[x as usize][y as usize] == TileType::Water
            {
                let mut pond_tiles = Vec::new();
                flood_fill_water(tiles, &mut visited, x, y, map_size, &mut pond_tiles);

                if !pond_tiles.is_empty() {
                    // Calculate center point
                    let center = pond_tiles.iter().fold(Vec2::ZERO, |acc, &(x, y)| {
                        acc + Vec2::new(x as f32, y as f32)
                    }) / pond_tiles.len() as f32;

                    // Calculate radius based on furthest point
                    let radius = pond_tiles
                        .iter()
                        .map(|&(x, y)| Vec2::new(x as f32, y as f32).distance(center))
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(1.0);

                    ponds.push(WaterPond { center, radius });
                }
            }
        }
    }

    ponds
}

fn flood_fill_water(
    tiles: &[Vec<TileType>],
    visited: &mut Vec<Vec<bool>>,
    x: u32,
    y: u32,
    map_size: TilemapSize,
    pond_tiles: &mut Vec<(u32, u32)>,
) {
    if x >= map_size.x
        || y >= map_size.y
        || visited[x as usize][y as usize]
        || tiles[x as usize][y as usize] != TileType::Water
    {
        return;
    }

    visited[x as usize][y as usize] = true;
    pond_tiles.push((x, y));

    // Check all 8 directions for connected water tiles
    for dx in -1..=1 {
        for dy in -1..=1 {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;

            if new_x >= 0 && new_y >= 0 {
                flood_fill_water(
                    tiles,
                    visited,
                    new_x as u32,
                    new_y as u32,
                    map_size,
                    pond_tiles,
                );
            }
        }
    }
}

fn spawn_water_pond_colliders(
    commands: &mut Commands,
    ponds: &[WaterPond],
    tile_size: &TileSize,
    offset: Vec2,
) {
    for pond in ponds {
        let pos = Vec2::new(
            pond.center.x * tile_size.x + tile_size.x / 2.0,
            pond.center.y * tile_size.y + tile_size.y / 2.0,
        );

        // Apply the offset to center the collider
        let adjusted_pos = pos + offset;

        // Calculate circle diameter based on radius and tile size
        let diameter = pond.radius * 2.0 * tile_size.x.min(tile_size.y);

        commands.spawn((
            Water,
            Collider::circle(diameter / 2.0),
            RigidBody::Static,
            CollisionLayers::new(
                GameCollisionLayer::Water,
                [
                    GameCollisionLayer::Player,
                    GameCollisionLayer::Npc,
                    GameCollisionLayer::Enemy,
                ],
            ),
            Transform::from_xyz(adjusted_pos.x, adjusted_pos.y, 1.0),
            GlobalTransform::default(),
        ));
    }
}
