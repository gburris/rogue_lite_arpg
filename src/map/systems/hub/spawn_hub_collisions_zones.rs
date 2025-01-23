use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    helpers::labels::GameCollisionLayer,
    map::{
        components::{MapLayout, TileType, Wall, Water},
        resources::TileSize,
        WallSection, WorldSpaceConfig,
    },
};

pub fn spawn_hub_colliders(
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
    start: Vec2,
    end: Vec2,
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
                    // Calculate bounding box
                    let min_x = pond_tiles.iter().map(|&(x, _)| x).min().unwrap() as f32;
                    let max_x = pond_tiles.iter().map(|&(x, _)| x).max().unwrap() as f32;
                    let min_y = pond_tiles.iter().map(|&(_, y)| y).min().unwrap() as f32;
                    let max_y = pond_tiles.iter().map(|&(_, y)| y).max().unwrap() as f32;

                    // Determine the orientation of the capsule (horizontal or vertical)
                    let width = max_x - min_x + 1.0; // +1 to include the last tile
                    let height = max_y - min_y + 1.0; // +1 to include the last tile

                    let (start, end, radius) = if width > height {
                        // Horizontal capsule
                        let start = Vec2::new(min_x, (min_y + max_y) / 2.0);
                        let end = Vec2::new(max_x, (min_y + max_y) / 2.0);
                        let radius = height / 2.0; // Radius is half the height
                        (start, end, radius)
                    } else {
                        // Vertical capsule
                        let start = Vec2::new((min_x + max_x) / 2.0, min_y);
                        let end = Vec2::new((min_x + max_x) / 2.0, max_y);
                        let radius = width / 2.0; // Radius is half the width
                        (start, end, radius)
                    };

                    ponds.push(WaterPond { start, end, radius });
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
        // Convert tile coordinates to world coordinates
        let start_pos = Vec2::new(pond.start.x * tile_size.x, pond.start.y * tile_size.y);

        let end_pos = Vec2::new(pond.end.x * tile_size.x, pond.end.y * tile_size.y);

        // Calculate the length of the capsule in world units
        let length = start_pos.distance(end_pos);

        // Calculate the radius in world units
        let radius = pond.radius * tile_size.x.min(tile_size.y);

        // Calculate the center position of the capsule in world units
        let center_pos = (start_pos + end_pos) / 2.0;

        // Apply the offset to center the collider
        let adjusted_pos = center_pos + offset;
        // Determine if the capsule is horizontal or vertical
        let is_horizontal = pond.start.y == pond.end.y;

        // Create a transform for the capsule
        let mut transform = Transform::from_xyz(adjusted_pos.x, adjusted_pos.y, 1.0);

        // Rotate the capsule if it's horizontal
        if is_horizontal {
            transform.rotate_z(std::f32::consts::FRAC_PI_2); // Rotate 90 degrees (π/2 radians)
        }

        // Spawn the capsule collider
        commands.spawn((
            Water,
            Collider::capsule(radius, length),
            RigidBody::Static,
            CollisionLayers::new(
                GameCollisionLayer::Water,
                [
                    GameCollisionLayer::Player,
                    GameCollisionLayer::Npc,
                    GameCollisionLayer::Enemy,
                ],
            ),
            transform,
            GlobalTransform::default(),
        ));
    }
}
