mod generation;
mod instance;
mod prefabs;
mod space;
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
        map::{
            generation::{MapData, MapDataBuilder},
            space::{TileGrid, WorldSpaceConfig},
        },
    },
};

pub mod prelude {
    pub use super::instance::*;
    pub use super::prefabs::*;
    pub use super::space::WorldSpaceConfig;
    pub use super::zone::*;
    pub use super::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((instance::plugin, zone::plugin))
        .add_systems(OnEnter(AppState::CreateHub), insert_hub_layout)
        .insert_resource(WorldSpaceConfig::default());
}

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

impl TileType {
    pub fn blocks_ground(&self) -> bool {
        match self {
            TileType::Wall | TileType::Water => true,
            _ => false,
        }
    }

    pub fn should_render(&self) -> bool {
        match self {
            TileType::DeadZone => false,
            _ => true,
        }
    }

    pub fn can_spawn(&self) -> bool {
        !self.blocks_ground() && self.should_render()
    }
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

#[derive(Resource, Clone)]
pub struct MapLayout {
    pub size: TilemapSize,
    pub markers: MapMarkers,
    pub environmental_colliders: Vec<EnvironmentalMapCollider>,
    pub grid: TileGrid,
}

impl From<MapData> for MapLayout {
    fn from(map_data: MapData) -> Self {
        MapLayout {
            size: map_data.size,
            markers: MapMarkers {
                markers: map_data.markers,
            },
            environmental_colliders: map_data.colliders,
            grid: map_data.grid,
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

    let map_data = MapDataBuilder::new(map_size, TileType::Grass)
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

    fn render(grid: &TileGrid) -> String {
        let width = grid.map_size().x as usize;
        let height = grid.map_size().y as usize;
        if width == 0 || height == 0 {
            return String::new();
        }

        (0..height)
        .rev()                                      // top row first
        .map(|y| {
            (0..width)
                .map(|x| {
                    let tile = grid.get(x as i32, y as i32).unwrap_or(TileType::DeadZone);
                    to_char(&tile)
                })
                .collect::<String>()                // collect row chars into a String
        })
        .collect::<Vec<String>>()                   // collect all rows
        .join("\n")                                 // join with newline
        + "\n"
    }

    #[test]
    fn hub_layout_matches_snapshot() {
        // see isn't used for hub, so if it becomes seed-sensitive we want to know
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let map_size = TilemapSize { x: 100, y: 100 };
        let hub_layout = MapDataBuilder::new(map_size, TileType::Grass)
            .with_exterior_walls()
            .with_exits(0)
            .with_prefab(PrefabType::NPCHub)
            .build(&mut rng);
        let rendered_hub_layout = render(&hub_layout.grid);

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
        let hub_layout1 = MapDataBuilder::new(map_size, TileType::Grass)
            .with_exterior_walls()
            .with_exits(0)
            .with_prefab(PrefabType::NPCHub)
            .build(&mut rng1);
        let hub_layout2 = MapDataBuilder::new(map_size, TileType::Grass)
            .with_exterior_walls()
            .with_exits(0)
            .with_prefab(PrefabType::NPCHub)
            .build(&mut rng2);

        let rendered_layout_1 = render(&hub_layout1.grid);
        let rendered_layout_2 = render(&hub_layout2.grid);
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
