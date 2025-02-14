use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::TilePos,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Component, Clone, Default)]
pub struct Mapper {
    pub map_layout: MapLayout,
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Water;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Wood,
    Ground,
    Wall,
    Water,
    Cobblestone,
    DeadZone, //Marker for DO NOT RENDER for empty space in the map
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MarkerType {
    PlayerSpawn,
    LevelExit,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultiMarkerType {
    EnemySpawns,
    BossSpawns,
    ChestSpawns,
    NPCSpawns,
}

#[derive(Clone, Default, Debug)]
pub struct MapMarkers {
    pub single_markers: HashMap<MarkerType, Vec2>, // Single-instance markers
    pub multi_markers: HashMap<MultiMarkerType, Vec<Vec2>>, // Multi-instance markers
}

//Code to actually get the marker
impl MapMarkers {
    pub fn get_single(&self, marker_type: MarkerType) -> Option<&Vec2> {
        self.single_markers.get(&marker_type)
    }

    pub fn get_multi(&self, marker_type: MultiMarkerType) -> Option<&Vec<Vec2>> {
        self.multi_markers.get(&marker_type)
    }
}

#[derive(Debug, Clone)]
pub enum EnvironmentalType {
    Wall,
    // Water, // Commented out for now as requested
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
    pub map_size: TilemapSize,
    pub tile_size: TilemapTileSize, // Size of each tile in world units
    pub world_origin: Vec2,         // Where (0,0) in tile coordinates maps to in world space
}

//If we want to f with the scale of our tiles:world (e.g. have way more tiles in our world)
//We can edit that here
impl Default for WorldSpaceConfig {
    fn default() -> Self {
        WorldSpaceConfig {
            map_size: TilemapSize::new(33, 33),
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
pub struct InstanceConfig {
    pub instances: HashMap<String, InstanceType>,
}

#[derive(Deserialize, Debug)]
pub struct InstanceType {
    pub size_x_range: (f32, f32),
    pub size_y_range: (f32, f32),
    pub number_of_enemies_range: (f32, f32),
    pub dead_zone_squares: bool,
    pub chest_range: (f32, f32),
}

#[derive(Resource)]
pub struct InstanceAssets {
    pub instance_config: HashMap<String, InstanceType>,
}
