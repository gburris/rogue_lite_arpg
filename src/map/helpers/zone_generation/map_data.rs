use bevy::{math::Vec2, transform::components::Transform};
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::components::{EnvironmentalMapCollider, EnvironmentalType, TileType};

use super::{
    dead_zone::add_dead_zones,
    hub::build_hub,
    utils::{calculate_center_rect, calculate_collider_position, calculate_wall_dimensions},
    walls::add_exterior_walls,
};

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

pub struct MapDataBuilder {
    map_data: MapData,
    size: TilemapSize,
    hub_size: Option<TilemapSize>,
    should_add_dead_zones: bool,
    floor_type: TileType,
}

impl MapDataBuilder {
    pub fn new(size: TilemapSize) -> Self {
        Self {
            map_data: MapData::new(size),
            size,
            hub_size: None,
            should_add_dead_zones: false,
            floor_type: TileType::Ground,
        }
    }

    pub fn with_grass_floor(mut self) -> Self {
        self.floor_type = TileType::Grass;
        self.map_data = MapData::new_with_grass(self.size);
        self
    }

    pub fn with_exterior_walls(mut self) -> Self {
        add_exterior_walls(&mut self.map_data, self.size);
        self
    }

    pub fn with_dead_zones(mut self) -> Self {
        self.should_add_dead_zones = true;
        self
    }

    pub fn with_hub(mut self, hub_size: TilemapSize) -> Self {
        self.hub_size = Some(hub_size);
        self
    }

    pub fn build(mut self) -> MapData {
        // Add dead zones if requested
        if self.should_add_dead_zones {
            add_dead_zones(&mut self.map_data, self.size);
        }

        // Add hub if specified
        if let Some(hub_size) = self.hub_size {
            let hub_bounds = calculate_center_rect(self.size, hub_size);
            build_hub(&mut self.map_data, &hub_bounds);
        }

        // Convert MapData to MapLayout
        MapData {
            tiles: self.map_data.tiles,
            colliders: self.map_data.colliders,
        }
    }
}
