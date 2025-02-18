use bevy::{math::Vec2, transform::components::Transform};
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::components::{EnvironmentalMapCollider, EnvironmentalType, TileType};

use super::utils::{calculate_collider_position, calculate_wall_dimensions};

pub struct MapData {
    pub tiles: Vec<Vec<TileType>>,
    pub colliders: Vec<EnvironmentalMapCollider>,
}

impl MapData {
    pub fn new(size: TilemapSize) -> Self {
        Self {
            tiles: vec![vec![TileType::Ground; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
        }
    }
    pub fn new_with_grass(size: TilemapSize) -> Self {
        Self {
            tiles: vec![vec![TileType::Grass; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
        }
    }
    pub fn add_wall_collider(&mut self, start: (u32, u32), is_horizontal: bool, length: u32) {
        let start_pos = Vec2::new(start.0 as f32, start.1 as f32);
        let length = length as f32;

        let (width, height) = calculate_wall_dimensions(is_horizontal, length);
        let collider_pos = calculate_collider_position(start_pos, width, height, is_horizontal);

        self.colliders.push(EnvironmentalMapCollider {
            collider_type: EnvironmentalType::Wall,
            transform: Transform::from_xyz(collider_pos.x, collider_pos.y, 1.0),
            width,
            height,
        });
    }
}
