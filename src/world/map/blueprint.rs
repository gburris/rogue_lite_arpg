use std::collections::HashMap;
use std::fmt;

use bevy::prelude::*;

use super::tiles::{TileGrid, TileKind};
use crate::prelude::*;

/// Generated map tiles and placements, plus the replay key (`recipe`, `seed`) that built them.
#[derive(Component, Debug, PartialEq, Eq)]
#[require(Transform, Visibility, Name::new("Map"))]
pub struct MapBlueprint {
    /// Tile grid containing the map terrain.
    pub tiles: TileGrid,
    /// Entity placements associated with this map.
    pub placements: Vec<Placement>,
    /// Name of the recipe that built this map.
    pub recipe: String,
    /// Seed that drove the whole build: layout, placements and visuals.
    pub seed: u64,
}

impl MapBlueprint {
    /// Creates a map with a blocking wall border.
    pub fn new(map_size: UVec2, floor: TileKind, recipe: String, seed: u64) -> Self {
        let mut tiles = TileGrid::new(map_size, floor);

        // For now every map has a wall border
        tiles.fill_border(TileKind::Wall);

        Self {
            tiles,
            placements: Vec::new(),
            recipe,
            seed,
        }
    }

    /// Returns the sole player start, or an error if it is missing or duplicated.
    pub fn player_start(&self) -> Result<IVec2, MapValidationError> {
        let starts: Vec<_> = self
            .placements
            .iter()
            .filter_map(|placement| {
                matches!(placement.kind, PlacementKind::PlayerStart).then_some(placement.tile)
            })
            .collect();
        match starts.as_slice() {
            [start] => Ok(*start),
            [] => Err(MapValidationError::MissingPlayerStart),
            _ => Err(MapValidationError::MultiplePlayerStarts(starts.len())),
        }
    }

    /// Checks required placements, map boundaries, and placement tile validity.
    pub fn validate(&self) -> Result<(), MapValidationError> {
        self.player_start()?;
        if !self
            .placements
            .iter()
            .any(|placement| matches!(placement.kind, PlacementKind::Exit))
        {
            return Err(MapValidationError::MissingExit);
        }
        for (tile_position, tile_kind) in self.tiles.iter() {
            let map_size = self.map_size().as_ivec2();
            if (tile_position.x == 0
                || tile_position.y == 0
                || tile_position.x == map_size.x - 1
                || tile_position.y == map_size.y - 1)
                && !tile_kind.blocks_ground()
            {
                return Err(MapValidationError::NonBlockingGroundBoundary {
                    tile: tile_position,
                    tile_kind,
                });
            }
        }
        let mut occupied = HashMap::<IVec2, usize>::new();
        for (placement_index, placement) in self.placements.iter().enumerate() {
            let Some(tile_kind) = self.tiles.get(placement.tile) else {
                return Err(MapValidationError::PlacementOutOfBounds {
                    placement_index,
                    tile: placement.tile,
                });
            };
            if !tile_kind.can_spawn() {
                return Err(MapValidationError::PlacementNotSpawnable {
                    placement_index,
                    tile: placement.tile,
                    tile_kind,
                });
            }
            if let Some(previous_placement_index) = occupied.insert(placement.tile, placement_index)
            {
                return Err(MapValidationError::OverlappingPlacements {
                    tile: placement.tile,
                    placement_indices: [previous_placement_index, placement_index],
                });
            }
        }
        Ok(())
    }

    /// Returns the dimensions of this map in tiles.
    pub fn map_size(&self) -> UVec2 {
        self.tiles.map_size()
    }
}

/// Describes why a generated map failed validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapValidationError {
    MissingPlayerStart,
    MultiplePlayerStarts(usize),
    MissingExit,
    /// A map-edge tile is traversable; the entire outer ring must block ground movement.
    NonBlockingGroundBoundary {
        tile: IVec2,
        tile_kind: TileKind,
    },
    PlacementOutOfBounds {
        placement_index: usize,
        tile: IVec2,
    },
    PlacementNotSpawnable {
        placement_index: usize,
        tile: IVec2,
        tile_kind: TileKind,
    },
    OverlappingPlacements {
        tile: IVec2,
        placement_indices: [usize; 2],
    },
}

impl fmt::Display for MapValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPlayerStart => write!(f, "map is missing its player start"),
            Self::MultiplePlayerStarts(count) => {
                write!(f, "map has {count} player starts, expected exactly one")
            }
            Self::MissingExit => write!(f, "map is missing its exit"),
            Self::NonBlockingGroundBoundary { tile, tile_kind } => write!(
                f,
                "map border tile {tile} is {tile_kind:?}, which does not block ground movement"
            ),
            Self::PlacementOutOfBounds {
                placement_index,
                tile,
            } => write!(
                f,
                "placement {placement_index} at {tile} lies outside the map"
            ),
            Self::PlacementNotSpawnable {
                placement_index,
                tile,
                tile_kind,
            } => write!(
                f,
                "placement {placement_index} at {tile} sits on {tile_kind:?}, which cannot host a \
                 spawn"
            ),
            Self::OverlappingPlacements {
                tile,
                placement_indices,
            } => write!(
                f,
                "placements {} and {} overlap on tile {tile}",
                placement_indices[0], placement_indices[1]
            ),
        }
    }
}
impl std::error::Error for MapValidationError {}
/// An error raised while generating or validating a map.
#[derive(Debug)]
pub enum MapError {
    Generation(String),
    Validation(MapValidationError),
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generation(message) => f.write_str(message),
            Self::Validation(error) => write!(f, "map validation failed: {error}"),
        }
    }
}

impl std::error::Error for MapError {}

impl From<String> for MapError {
    fn from(message: String) -> Self {
        Self::Generation(message)
    }
}

impl From<&str> for MapError {
    fn from(message: &str) -> Self {
        Self::Generation(message.to_owned())
    }
}

/// A placed map entity and its tile coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub tile: IVec2,
    /// Entity kind represented by this placement.
    pub kind: PlacementKind,
}

/// Kind of entity placed in a map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementKind {
    PlayerStart,
    Enemy(EnemyKind),
    Npc(NpcKind),
    Chest,
    Exit,
}

impl PlacementKind {
    /// Converts a placement glyph to its entity kind, if recognized.
    pub const fn from_glyph(glyph: char) -> Option<Self> {
        match glyph {
            'P' => Some(Self::PlayerStart),
            'X' => Some(Self::Exit),
            'C' => Some(Self::Chest),
            'S' => Some(Self::Npc(NpcKind::Shopkeeper)),
            'H' => Some(Self::Npc(NpcKind::Helper)),
            'T' => Some(Self::Npc(NpcKind::StatTrainer)),
            'W' => Some(Self::Enemy(EnemyKind::Warrior)),
            'F' => Some(Self::Enemy(EnemyKind::FireMage)),
            'I' => Some(Self::Enemy(EnemyKind::IceMage)),
            'B' => Some(Self::Enemy(EnemyKind::Bat)),
            _ => None,
        }
    }
}
