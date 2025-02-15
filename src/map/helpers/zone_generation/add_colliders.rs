use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::{components::TileType, EnvironmentalMapCollider, EnvironmentalType, WallSection};

pub fn add_environmental_colliders_to_zone(
    tiles: &[Vec<TileType>],
    map_size: TilemapSize,
) -> Vec<EnvironmentalMapCollider> {
    let mut colliders = Vec::new();
    let wall_sections = find_wall_sections(tiles, map_size);

    for section in wall_sections {
        let start_pos = Vec2::new(section.start.0 as f32, section.start.1 as f32);

        let length = section.length() as f32;

        let (width, height) = if section.is_horizontal {
            (length, 1.0)
        } else {
            (1.0, length)
        };

        let collider_pos = if section.is_horizontal {
            Vec2::new(start_pos.x + (width / 2.0), start_pos.y + 0.5)
        } else {
            Vec2::new(start_pos.x + 0.5, start_pos.y + (height / 2.0))
        };

        colliders.push(EnvironmentalMapCollider {
            collider_type: EnvironmentalType::Wall,
            transform: Transform::from_xyz(collider_pos.x, collider_pos.y, 1.0),
            width,
            height,
        });
    }

    colliders
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
