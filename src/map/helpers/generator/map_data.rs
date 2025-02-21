use bevy::{log::warn, math::Vec2, transform::components::Transform};
use bevy_ecs_tilemap::map::TilemapSize;
use std::collections::HashMap;

use crate::map::components::{EnvironmentalMapCollider, EnvironmentalType, MarkerType, TileType};

use super::{
    dead_zone::add_dead_zones,
    hub::{build_hub, get_hub_markers},
    temple::{build_temple, get_temple_markers},
    utils::{
        calculate_collider_position, calculate_wall_dimensions, find_multiple_positions,
        generate_entrance_exit_positions,
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
    pub fn new(size: TilemapSize, floor_type: TileType) -> Self {
        Self {
            size,
            tiles: vec![vec![floor_type; size.y as usize]; size.x as usize],
            colliders: Vec::new(),
            markers: HashMap::new(),
        }
    }

    // Updates all ground tiles to the new floor type while preserving other tile types
    pub fn set_floor(&mut self, floor_type: TileType) {
        for row in self.tiles.iter_mut() {
            for tile in row.iter_mut() {
                *tile = floor_type;
            }
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

pub enum Prefab {
    NPCHub,
    Temple,
}

pub struct MapDataBuilder {
    map_data: MapData,
    size: TilemapSize,
    prefabs: Vec<Prefab>,
    should_add_dead_zones: bool,
    num_enemies: Option<u32>,
    num_chests: Option<u32>,
}

impl MapDataBuilder {
    pub fn new(size: TilemapSize) -> Self {
        Self {
            map_data: MapData::new(size, TileType::Ground), // Default to ground
            size,
            prefabs: Vec::new(),
            should_add_dead_zones: false,
            num_enemies: None,
            num_chests: None,
        }
    }

    pub fn with_floor(mut self, floor_type: TileType) -> Self {
        self.map_data.set_floor(floor_type);
        self
    }

    pub fn with_prefab(mut self, prefab: Prefab) -> Self {
        self.prefabs.push(prefab);
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

    fn generate_random_markers(&self) -> HashMap<MarkerType, Vec<Vec2>> {
        let mut markers = HashMap::new();

        if let Some(num_enemies) = self.num_enemies {
            let enemy_positions =
                find_multiple_positions(&self.map_data.tiles, self.size, 0.3..0.7, num_enemies);
            markers.insert(MarkerType::EnemySpawns, enemy_positions);
        }

        if let Some(num_chests) = self.num_chests {
            let chest_positions =
                find_multiple_positions(&self.map_data.tiles, self.size, 0.2..0.8, num_chests);
            markers.insert(MarkerType::ChestSpawns, chest_positions);
        }

        // Always generate entrance/exit positions for random layouts
        let (player_pos, exit_positions) = generate_entrance_exit_positions(self.size);
        markers.insert(MarkerType::PlayerSpawns, player_pos);
        markers.insert(MarkerType::LevelExits, exit_positions);

        markers
    }

    pub fn build(mut self) -> MapData {
        if self.should_add_dead_zones {
            add_dead_zones(&mut self.map_data, self.size);
        }

        for prefab in &self.prefabs {
            match prefab {
                Prefab::Temple => {
                    if let Some(template_rectangle) = build_temple(&mut self.map_data) {
                        let temple_markers: HashMap<MarkerType, Vec<Vec2>> =
                            get_temple_markers(&template_rectangle);
                        merge_markers(&mut self.map_data.markers, temple_markers);
                    } else {
                        warn!("No temple markers -> Temple was not returned");
                    }
                }
                Prefab::NPCHub => {
                    let hub_size = build_hub(&mut self.map_data);
                    merge_markers(
                        &mut self.map_data.markers,
                        get_hub_markers(self.size, hub_size),
                    );
                }
            }
        }
        //Add all other map markers
        let random_markers = self.generate_random_markers();
        merge_markers(&mut self.map_data.markers, random_markers);

        self.map_data
    }
}

fn merge_markers(
    existing_markers: &mut HashMap<MarkerType, Vec<Vec2>>,
    new_markers: HashMap<MarkerType, Vec<Vec2>>,
) {
    for (marker_type, positions) in new_markers {
        existing_markers
            .entry(marker_type)
            .or_insert_with(Vec::new)
            .extend(positions);
    }
}
