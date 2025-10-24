mod instance;
mod map_data;
mod prefabs;
mod utils;
mod walls;
mod zone;

use std::collections::HashMap;

use ::bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    prelude::{AppState, PrefabType},
    world::map::map_data::{MapData, MapDataBuilder},
};

pub mod prelude {
    pub use super::instance::*;
    pub use super::prefabs::*;
    pub use super::zone::*;
    pub use super::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((instance::plugin, zone::plugin))
        .add_systems(OnEnter(AppState::CreateHub), insert_hub_layout)
        .insert_resource(WorldSpaceConfig::default());
}

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
        let low = TilePos::new(0, 0).center_in_world(
            &map_size_in_tiles,
            &grid_size,
            &self.tile_size,
            &map_type,
            &TilemapAnchor::Center,
        );
        let high = TilePos::new(map_size_in_tiles.x, map_size_in_tiles.y).center_in_world(
            &map_size_in_tiles,
            &grid_size,
            &self.tile_size,
            &map_type,
            &TilemapAnchor::Center,
        );
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

fn insert_hub_layout(mut commands: Commands, mut game_state: ResMut<NextState<AppState>>) {
    let map_size = TilemapSize { x: 100, y: 100 };

    let map_data = MapDataBuilder::new(map_size)
        .with_floor(TileType::Grass)
        .with_exterior_walls()
        .with_exits(0)
        .with_prefab(PrefabType::NPCHub)
        .build();
    let map_layout = MapLayout::from(map_data);

    commands.insert_resource(map_layout);
    game_state.set(AppState::SpawnZone);
}
