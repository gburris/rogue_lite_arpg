use bevy::{log::warn, math::Vec2, prelude::*};
use bevy_ecs_tilemap::map::TilemapSize;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

use crate::{
    prelude::{EmptySquare, Hub, Prefab, PrefabType, Temple},
    world::map::{
        EnvironmentalMapCollider, EnvironmentalType, MarkerType, TileType, space::TileGrid,
        walls::add_exterior_walls,
    },
};

use super::utils::{
    calculate_collider_position, calculate_wall_dimensions, find_multiple_positions,
    generate_entrance_exit_positions,
};

pub struct MapData {
    pub size: TilemapSize,
    pub grid: TileGrid,
    pub colliders: Vec<EnvironmentalMapCollider>,
    pub markers: HashMap<MarkerType, Vec<Vec2>>,
}

impl MapData {
    pub fn new(size: TilemapSize, fill: TileType) -> Self {
        Self {
            size,
            grid: TileGrid::new(size.into(), fill),
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

pub struct MapDataBuilder {
    map_data: MapData,
    size: TilemapSize,
    prefabs: Vec<PrefabType>,
    num_enemies: Option<u32>,
    num_exits: u32,
    num_chests: Option<u32>,
}

impl MapDataBuilder {
    pub fn new(size: TilemapSize, fill: TileType) -> Self {
        Self {
            map_data: MapData::new(size, fill),
            size,
            prefabs: Vec::new(),
            num_enemies: None,
            num_chests: None,
            num_exits: 0,
        }
    }

    pub fn with_prefab(mut self, prefab: PrefabType) -> Self {
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

    pub fn with_exits(mut self, count: u32) -> Self {
        self.num_exits = count;
        self
    }

    pub fn with_exterior_walls(mut self) -> Self {
        add_exterior_walls(&mut self.map_data, self.size);
        self
    }

    fn generate_random_markers(&self, rng: &mut ChaCha8Rng) -> HashMap<MarkerType, Vec<Vec2>> {
        let mut markers = HashMap::new();

        if let Some(num_enemies) = self.num_enemies {
            let enemy_positions =
                find_multiple_positions(&self.map_data.grid, 0.3..0.7, num_enemies, rng);
            markers.insert(MarkerType::EnemySpawns, enemy_positions);
        }

        if let Some(num_chests) = self.num_chests {
            let chest_positions =
                find_multiple_positions(&self.map_data.grid, 0.2..0.8, num_chests, rng);
            markers.insert(MarkerType::ChestSpawns, chest_positions);
        }

        let has_player_spawn = self
            .map_data
            .markers
            .get(&MarkerType::PlayerSpawns)
            .is_some_and(|pos| !pos.is_empty());

        // Always generate entrance/exit positions for random sprite_layouts
        let (player_pos, exit_positions) =
            generate_entrance_exit_positions(self.map_data.grid.map_size(), self.num_exits, rng);

        if !has_player_spawn {
            info!(
                "New player position determined: {}",
                player_pos.first().unwrap()
            );
            markers.insert(MarkerType::PlayerSpawns, player_pos);
        }
        markers.insert(MarkerType::LevelExits, exit_positions);

        markers
    }

    pub fn build(mut self, rng: &mut ChaCha8Rng) -> MapData {
        for prefab_type in &self.prefabs {
            let prefab: Box<dyn Prefab> = match prefab_type {
                PrefabType::Temple => Box::new(Temple),
                PrefabType::NPCHub => Box::new(Hub),
                PrefabType::EmptySquare => Box::new(EmptySquare),
            };

            if let Some(bounds) = prefab.build(&mut self.map_data) {
                let markers = prefab.get_markers(&bounds);
                merge_markers(&mut self.map_data.markers, markers);
            } else {
                warn!("Failed to build prefab: ");
            }
        }
        //Add all other map markers
        let random_markers = self.generate_random_markers(rng);
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
            .or_default()
            .extend(positions);
    }
}
