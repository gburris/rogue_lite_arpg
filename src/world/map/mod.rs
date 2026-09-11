mod blueprint;
mod collisions;
mod coordinates;
mod generation;
mod instance;
mod prefabs;
mod spawn;
mod walls;

use ::bevy::prelude::*;
use serde::Deserialize;

use crate::{
    prelude::AppState,
    world::{
        MapRng,
        map::{blueprint::MapBlueprint, coordinates::MapCoordinates, prefabs::build_hub},
    },
};

pub mod prelude {
    pub use super::blueprint::MapBlueprint;
    pub use super::coordinates::MapCoordinates;
    pub use super::instance::*;
    pub use super::prefabs::*;
    pub use super::spawn::*;
    pub use super::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((instance::plugin, spawn::plugin))
        .add_systems(OnEnter(AppState::CreateHub), generate_hub_layout)
        .insert_resource(MapCoordinates::default());
}

#[derive(Clone, Eq, Hash, Copy, PartialEq, Deserialize, Debug)]
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
        matches!(self, Self::Wall | Self::DeadZone)
    }

    pub fn should_render(&self) -> bool {
        !matches!(self, Self::DeadZone)
    }

    pub fn can_spawn(&self) -> bool {
        matches!(
            self,
            Self::Ground | Self::Cobblestone | Self::Grass | Self::Wood
        )
    }

    pub fn ascii_char(&self) -> char {
        match self {
            Self::Wood => 'w',
            Self::Ground => '.',
            Self::Grass => ',',
            Self::Wall => '#',
            Self::Water => '~',
            Self::Cobblestone => 'c',
            Self::DeadZone => '_',
        }
    }
}

const HUB_MAP_SIZE: UVec2 = UVec2::new(100, 100);

fn generate_hub_layout(
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    mut rng: ResMut<MapRng>,
) {
    let (_, zone_seed) = rng.zone();
    info!("Generating hub zone with seed: {}", zone_seed);

    let mut blueprint = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, zone_seed);
    build_hub(&mut blueprint);

    commands.insert_resource(blueprint);
    game_state.set(AppState::SpawnZone);
}

#[cfg(test)]
mod tests {
    use crate::world::map::blueprint::TileGrid;

    use super::*;
    use rand::{Rng, SeedableRng};
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
        let width = grid.map_size().x as i32;
        let height = grid.map_size().y as i32;
        if width == 0 || height == 0 {
            return String::new();
        }

        (0..height)
        .rev()                                      // top row first
        .map(|y| {
            (0..width)
                .map(|x| {
                    let tile = grid.get(IVec2::new(x, y)).unwrap_or(TileType::DeadZone);
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
        let zone_seed: u64 = rng.next_u64();

        let mut blueprint = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, zone_seed);
        build_hub(&mut blueprint);
        let rendered_hub_layout = render(&blueprint.tiles);

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
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let zone_seed: u64 = rng.next_u64();
        let mut blueprint1 = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, zone_seed);
        let mut blueprint2 = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, zone_seed);

        build_hub(&mut blueprint1);
        build_hub(&mut blueprint2);

        let rendered_layout_1 = render(&blueprint1.tiles);
        let rendered_layout_2 = render(&blueprint2.tiles);

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
