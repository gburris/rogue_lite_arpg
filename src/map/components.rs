use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::TilePos,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::enemy::systems::enemy_spawn::EnemySpawnData;

use super::helpers::generator::MapData;

/*
MAP EVENTS - Should be the only part of map exposed to other crates
*/

#[derive(Event)]
pub struct SpawnZoneEvent;

#[derive(Event)]
pub struct CleanupZone;

#[derive(Event)]
pub struct NPCSpawnEvent(pub Vec<Vec2>);

#[derive(Debug, Event)]
pub struct EnemiesSpawnEvent(pub Vec<EnemySpawnData>);

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Water;

#[derive(Clone, Eq, Hash, Copy, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Wood,
    Ground,
    Grass,
    Wall,
    Water,
    Cobblestone,
    DeadZone, //Marker for DO NOT RENDER for empty space in the map
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MarkerType {
    EnemySpawns,
    BossSpawns,
    ChestSpawns,
    NPCSpawns,
    PlayerSpawns,
    LevelExits,
}

#[derive(Clone, Default, Debug)]
pub struct MapMarkers {
    pub markers: HashMap<MarkerType, Vec<Vec2>>,
}

impl MapMarkers {
    pub fn get_markers(&self, marker_type: MarkerType) -> Option<&Vec<Vec2>> {
        self.markers.get(&marker_type)
    }
}

#[derive(Debug, Clone)]
pub enum EnvironmentalType {
    Wall,
    Water,
}

#[derive(Debug, Clone)]
pub struct EnvironmentalMapCollider {
    pub collider_type: EnvironmentalType,
    pub transform: Transform,
    pub width: f32,
    pub height: f32,
}
#[derive(Resource, Default, Clone)]
pub struct MapLayout {
    pub size: TilemapSize,
    pub tiles: Vec<Vec<TileType>>,
    pub markers: MapMarkers,
    pub environmental_colliders: Vec<EnvironmentalMapCollider>,
}

impl From<MapData> for MapLayout {
    fn from(map_data: MapData) -> Self {
        MapLayout {
            size: map_data.size,
            tiles: map_data.tiles,
            markers: MapMarkers {
                markers: map_data.markers,
            },
            environmental_colliders: map_data.colliders,
        }
    }
}
#[derive(Default)]
pub struct WallSection {
    pub start: (u32, u32),
    pub is_horizontal: bool,
    end: (u32, u32),
}

impl WallSection {
    pub fn new(start: (u32, u32), is_horizontal: bool) -> Self {
        WallSection {
            start,
            is_horizontal,
            end: start,
        }
    }

    pub fn extend(&mut self, pos: (u32, u32)) {
        self.end = pos;
    }

    pub fn length(&self) -> u32 {
        if self.is_horizontal {
            self.end.0 - self.start.0 + 1
        } else {
            self.end.1 - self.start.1 + 1
        }
    }
}

//This holds the concept of "Tiles are this big relative to world cordinaties"
#[derive(Resource)]
pub struct WorldSpaceConfig {
    pub tile_size: TilemapTileSize, // Size of each tile in world units
    pub world_origin: Vec2,         // Where (0,0) in tile coordinates maps to in world space
}

//If we want to f with the scale of our tiles:world (e.g. have way more tiles in our world)
//We can edit that here
impl Default for WorldSpaceConfig {
    fn default() -> Self {
        WorldSpaceConfig {
            tile_size: TilemapTileSize::new(32.0, 32.0),
            world_origin: Vec2::ZERO,
        }
    }
}

//Helper impl -> Pass in a tile, and it tells you the world co-ords it maps to
//This seems jank, but it's because the rendering of the tiles has this offset in it's
//Library and in rendering code it's used to "Center" the tiles onto the bevy map
impl WorldSpaceConfig {
    pub fn tile_to_world(&self, map_size_in_tiles: TilemapSize, tile_pos: IVec2) -> Vec2 {
        // Calculate the offset to center the tilemap
        let grid_size = TilemapGridSize::new(self.tile_size.x, self.tile_size.y);
        let map_type = TilemapType::Square;
        let low = TilePos::new(0, 0).center_in_world(&grid_size, &map_type);
        let high = TilePos::new(map_size_in_tiles.x, map_size_in_tiles.y)
            .center_in_world(&grid_size, &map_type);
        let diff = high - low;
        let offset = Vec2::new(-diff.x / 2.0, -diff.y / 2.0);

        // Compute world position with offset applied
        self.world_origin
            + Vec2::new(
                tile_pos.x as f32 * self.tile_size.x,
                tile_pos.y as f32 * self.tile_size.y,
            )
            + offset
    }
}

#[derive(Deserialize, Debug)]
pub struct InstanceType {
    pub size_x_range: (f32, f32),
    pub size_y_range: (f32, f32),
    pub number_of_enemies_range: (f32, f32),
    pub num_exits: u32,
    pub chest_range: (f32, f32),
    pub prefabs: Vec<String>,
    pub floor_type: String,
}

#[derive(Deserialize, Resource)]
pub struct InstanceAssets {
    pub instance_config: HashMap<String, InstanceType>,
}
