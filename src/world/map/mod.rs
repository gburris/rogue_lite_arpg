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
    world::{
        MapRng,
        map::map_data::{MapData, MapDataBuilder},
    },
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

#[derive(Clone, Eq, Hash, Copy, PartialEq, Serialize, Deserialize, Debug)]
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

fn insert_hub_layout(
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    mut rng: ResMut<MapRng>,
) {
    let map_size = TilemapSize { x: 100, y: 100 };

    let (mut zone_rng, zone_seed) = rng.zone();
    info!("Generating hub zone with seed: {}", zone_seed);

    let map_data = MapDataBuilder::new(map_size)
        .with_floor(TileType::Grass)
        .with_exterior_walls()
        .with_exits(0)
        .with_prefab(PrefabType::NPCHub)
        .build(&mut zone_rng);
    let map_layout = MapLayout::from(map_data);

    commands.insert_resource(map_layout);
    game_state.set(AppState::SpawnZone);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    const SNAPSHOT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/hub_layout.txt"
    );

    fn to_char(tile: &TileType) -> char {
        match tile {
            TileType::Wood => 'w',
            TileType::Ground => '.',
            TileType::Grass => ',',
            TileType::Wall => '#',
            TileType::Water => '~',
            TileType::Cobblestone => 'c',
            TileType::DeadZone => '_',
        }
    }

    fn render(tiles: &[Vec<TileType>]) -> String {
        let Some(first) = tiles.first() else {
            return String::new();
        };
        let height = first.len();
        assert!(
            tiles.iter().all(|col| col.len() == first.len()),
            "All columns must have the same length"
        );

        let mut output = (0..height)
            .rev()
            .map(|y| tiles.iter().map(|col| to_char(&col[y])).collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        output.push('\n');
        output
    }

    #[test]
    fn hub_layout_matches_snapshot() {
        // see isn't used for hub, so if it becomes seed-sensitive we want to know
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let map_size = TilemapSize { x: 100, y: 100 };
        let hub_layout = MapDataBuilder::new(map_size)
            .with_floor(TileType::Grass)
            .with_exterior_walls()
            .with_exits(0)
            .with_prefab(PrefabType::NPCHub)
            .build(&mut rng);
        let rendered_hub_layout = render(&hub_layout.tiles);

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(SNAPSHOT_PATH, &rendered_hub_layout).expect("Failed to write _PATH");
            return;
        }

        let expected_hub_layout =
            std::fs::read_to_string(SNAPSHOT_PATH).expect("Failed to read snapshot");
        assert_eq!(
            rendered_hub_layout,
            expected_hub_layout,
            "{}",
            first_difference(&expected_hub_layout, &rendered_hub_layout)
        );
    }

    #[test]
    fn hub_is_deterministic() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let map_size = TilemapSize { x: 100, y: 100 };
        let hub_layout1 = MapDataBuilder::new(map_size)
            .with_floor(TileType::Grass)
            .with_exterior_walls()
            .with_exits(0)
            .with_prefab(PrefabType::NPCHub)
            .build(&mut rng1);
        let hub_layout2 = MapDataBuilder::new(map_size)
            .with_floor(TileType::Grass)
            .with_exterior_walls()
            .with_exits(0)
            .with_prefab(PrefabType::NPCHub)
            .build(&mut rng2);

        let rendered_layout_1 = render(&hub_layout1.tiles);
        let rendered_layout_2 = render(&hub_layout2.tiles);
        assert_eq!(rendered_layout_1, rendered_layout_2);
    }

    fn first_difference(expected: &str, actual: &str) -> String {
        if expected.is_empty() {
            return format!(
                "no snapshot at {SNAPSHOT_PATH}, create one by running the test with UPDATE_SNAPSHOTS=1"
            );
        }

        let rows = actual.lines().count();

        for (row, (e_row, a_row)) in expected.lines().zip(actual.lines()).enumerate() {
            if e_row != a_row {
                let least_column_amount = e_row.chars().count().min(a_row.chars().count());

                let col = e_row
                    .chars()
                    .zip(a_row.chars())
                    .position(|(ec, ac)| ec != ac)
                    .unwrap_or(least_column_amount);

                let mut msg = format!(
                    "hub map changed at tile ({col}, {}) [snapshot line {}]\n",
                    rows - 1 - row,
                    row + 1,
                );
                msg.push_str(&format!("expected: {e_row}\n"));
                msg.push_str(&format!("actual:   {a_row}\n"));
                msg.push_str(&" ".repeat(12 + col));
                msg.push('^');
                msg.push('\n');
                return msg;
            }
        }

        format!(
            "hub map changed in size, expected {} rows, actual {} rows",
            expected.lines().count(),
            actual.lines().count()
        )
    }
}
