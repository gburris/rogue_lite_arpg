use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    helpers::labels::GameCollisionLayer,
    map::{
        components::{MapLayout, TileType, Wall},
        resources::TileSize,
        WallSection, WorldSpaceConfig,
    },
};

pub fn spawn_hub_collisions_zones(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
) {
    let map_size = world_config.map_size;
    //This should be casted to vec2 so we don't pass around the tilemap helper wrapper thing
    let tile_size = TileSize {
        x: world_config.tile_size.x,
        y: world_config.tile_size.y,
    };

    // Calculate center offset based on tilemap centering logic
    let grid_size = TilemapGridSize::new(world_config.tile_size.x, world_config.tile_size.y);
    let map_type = TilemapType::Square;

    let low = TilePos::new(0, 0).center_in_world(&grid_size, &map_type);
    let high = TilePos::new(map_size.x, map_size.y).center_in_world(&grid_size, &map_type);
    let diff = high - low;
    let offset = Vec2::new(-diff.x / 2.0, -diff.y / 2.0);

    // Process wall colliders
    let wall_sections = find_wall_sections(&map_layout.tiles, map_size);
    spawn_wall_colliders(&mut commands, &wall_sections, &tile_size, offset);

    // Process water ponds
    //TODO: Fix water collision on the pond
    //let water_ponds = find_water_ponds(&map_layout.tiles, map_size);
    //spawn_water_pond_colliders(&mut commands, &water_ponds, &tile_size, offset);
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
