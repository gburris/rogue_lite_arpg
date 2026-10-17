use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt;

use crate::prelude::{EnemyType, NpcType, TileGrid, TileType};

#[derive(Component, Debug, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub struct MapBlueprint {
    pub tiles: TileGrid,
    pub placements: Vec<Placement>,
    pub visual_seed: u64,
}

impl MapBlueprint {
    pub fn new(map_size: UVec2, floor: TileType, visual_seed: u64) -> Self {
        let mut tiles = TileGrid::new(map_size, floor);

        // For now every map has a wall border
        tiles.fill_border(TileType::Wall);

        Self {
            tiles,
            visual_seed,
            placements: Vec::new(),
        }
    }

    pub fn player_start(&self) -> Result<IVec2, MapValidationError> {
        let starts: Vec<_> = self
            .placements
            .iter()
            .filter_map(|p| matches!(p.kind, PlacementKind::PlayerStart).then_some(p.tile))
            .collect();
        match starts.as_slice() {
            [start] => Ok(*start),
            [] => Err(MapValidationError::MissingPlayerStart),
            _ => Err(MapValidationError::MultiplePlayerStarts(starts.len())),
        }
    }

    pub fn validate(&self) -> Result<(), MapValidationError> {
        self.player_start()?;
        let mut occupied = HashMap::<IVec2, usize>::new();
        for (index, placement) in self.placements.iter().enumerate() {
            let Some(tile_type) = self.tiles.get(placement.tile) else {
                return Err(MapValidationError::PlacementOutOfBounds {
                    index,
                    tile: placement.tile,
                });
            };
            if !tile_type.can_spawn() {
                info!("Failed attempting to place: {:?}", placement.kind);

                return Err(MapValidationError::PlacementNotSpawnable {
                    index,
                    tile: placement.tile,
                    tile_type,
                });
            }
            if let Some(previous) = occupied.insert(placement.tile, index) {
                return Err(MapValidationError::OverlappingPlacements {
                    tile: placement.tile,
                    indexes: [previous, index],
                });
            }
        }
        Ok(())
    }

    pub fn map_size(&self) -> UVec2 {
        self.tiles.map_size()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapValidationError {
    MissingPlayerStart,
    MultiplePlayerStarts(usize),
    PlacementOutOfBounds {
        index: usize,
        tile: IVec2,
    },
    PlacementNotSpawnable {
        index: usize,
        tile: IVec2,
        tile_type: TileType,
    },
    OverlappingPlacements {
        tile: IVec2,
        indexes: [usize; 2],
    },
}

impl fmt::Display for MapValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MapValidationError {}

pub type MapPlacement = Placement;
pub type MapPlacementKind = PlacementKind;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Placement {
    pub tile: IVec2,
    pub kind: PlacementKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PlacementKind {
    PlayerStart,
    Enemy(EnemyType),
    Npc(NpcType),
    Chest,
    Exit,
}

#[cfg(test)]
mod tests {
    use crate::world::map::prefabs::build_hub;

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

    fn render(tiles: &TileGrid) -> String {
        let width = tiles.map_size().x as usize;
        let height = tiles.map_size().y as usize;
        if width == 0 || height == 0 {
            return String::new();
        }

        (0..height)
        .rev()                                      // top row first
        .map(|y| {
            (0..width)
                .map(|x| {
                    let tile = tiles
                        .get(IVec2::new(x as i32, y as i32))
                        .unwrap_or(TileType::DeadZone);
                    to_char(&tile)
                })
                .collect::<String>()                // collect row chars into a String
        })
        .collect::<Vec<String>>()                   // collect all rows
        .join("\n")                                 // join with newline
        + "\n"
    }

    const HUB_MAP_SIZE: UVec2 = UVec2::new(100, 100);

    #[test]
    fn hub_layout_matches_snapshot() {
        // see isn't used for hub, so if it becomes seed-sensitive we want to know
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let map_seed: u64 = rng.next_u64();

        let mut blueprint = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, map_seed);
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

        let map_seed: u64 = rng.next_u64();
        let mut blueprint1 = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, map_seed);
        let mut blueprint2 = MapBlueprint::new(HUB_MAP_SIZE, TileType::Grass, map_seed);

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
