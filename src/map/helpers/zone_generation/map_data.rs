use bevy::{math::Vec2, transform::components::Transform};
use bevy_ecs_tilemap::map::TilemapSize;
use std::collections::HashMap;

use crate::map::components::{EnvironmentalMapCollider, EnvironmentalType, MarkerType, TileType};

use super::{
    dead_zone::add_dead_zones,
    hub::{build_hub, get_hub_markers},
    utils::{
        calculate_center_rect, calculate_collider_position, calculate_wall_dimensions,
        find_multiple_positions, generate_entrance_exit_positions,
    },
    walls::add_exterior_walls,
};

pub struct MapData {
    pub size: TilemapSize,
    pub tiles: Vec<Vec<TileType>>,
    pub colliders: Vec<EnvironmentalMapCollider>,
    pub markers: HashMap<MarkerType, Vec<Vec2>>,
}

impl MapData {
    pub fn new(size: TilemapSize) -> Self {
        Self {
            size,
            tiles: vec![vec![TileType::Ground; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
            markers: HashMap::new(),
        }
    }
    pub fn new_with_grass(size: TilemapSize) -> Self {
        Self {
            size,
            tiles: vec![vec![TileType::Grass; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
            markers: HashMap::new(),
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

pub enum MarkerPlacement {
    Random,
    Hub,
}

pub struct MapDataBuilder {
    map_data: MapData,
    size: TilemapSize,
    hub_size: Option<TilemapSize>,
    should_add_dead_zones: bool,
    floor_type: TileType,
    num_enemies: Option<u32>,
    num_chests: Option<u32>,
    marker_placement: Option<MarkerPlacement>,
}

impl MapDataBuilder {
    pub fn new(size: TilemapSize) -> Self {
        Self {
            map_data: MapData::new(size),
            size,
            hub_size: None,
            should_add_dead_zones: false,
            floor_type: TileType::Ground,
            num_enemies: None,
            num_chests: None,
            marker_placement: None,
        }
    }

    pub fn with_grass_floor(mut self) -> Self {
        self.floor_type = TileType::Grass;
        self.map_data = MapData::new_with_grass(self.size);
        self
    }
    pub fn with_enemies(mut self, count: u32) -> Self {
        self.num_enemies = Some(count);
        self
    }

    pub fn with_chests(mut self, count: u32) -> Self {
        self.num_chests = Some(count);
        self
    }
    pub fn with_exterior_walls(mut self) -> Self {
        add_exterior_walls(&mut self.map_data, self.size);
        self
    }

    pub fn with_dead_zones(mut self, include_dead_zones: bool) -> Self {
        self.should_add_dead_zones = include_dead_zones;
        self
    }

    pub fn with_hub(mut self, hub_size: TilemapSize) -> Self {
        self.hub_size = Some(hub_size);
        self
    }

    pub fn with_marker_placement(mut self, placement: MarkerPlacement) -> Self {
        self.marker_placement = Some(placement);
        self
    }

    fn generate_random_markers(&mut self) {
        if let Some(num_enemies) = self.num_enemies {
            let enemy_positions =
                find_multiple_positions(&self.map_data.tiles, self.size, 0.3..0.7, num_enemies);
            self.map_data
                .markers
                .insert(MarkerType::EnemySpawns, enemy_positions);
        }

        if let Some(num_chests) = self.num_chests {
            let chest_positions =
                find_multiple_positions(&self.map_data.tiles, self.size, 0.2..0.8, num_chests);
            self.map_data
                .markers
                .insert(MarkerType::ChestSpawns, chest_positions);
        }

        // Always generate entrance/exit positions for random layouts
        let (player_pos, exit_positions) = generate_entrance_exit_positions(self.size);
        self.map_data
            .markers
            .insert(MarkerType::PlayerSpawns, player_pos);
        self.map_data
            .markers
            .insert(MarkerType::LevelExits, exit_positions);
    }

    fn generate_hub_markers(&mut self) {
        let hub_size = self
            .hub_size
            .unwrap_or_else(|| panic!("Cannot add hub markers without a hub"));

        self.map_data.markers = get_hub_markers(self.size, hub_size);
    }

    pub fn build(mut self) -> MapData {
        // Add dead zones if requested
        if self.should_add_dead_zones {
            add_dead_zones(&mut self.map_data, self.size);
        }

        // Hub is now a "prefab" and can be added to any map
        if let Some(hub_size) = self.hub_size {
            let hub_bounds = calculate_center_rect(self.size, hub_size);
            build_hub(&mut self.map_data, &hub_bounds);
        }

        //Add markers based on user settings
        if let Some(ref placement) = self.marker_placement {
            match placement {
                MarkerPlacement::Random => {
                    self.generate_random_markers();
                }
                MarkerPlacement::Hub => {
                    self.generate_hub_markers();
                }
            }
        }
        self.map_data
    }
}
