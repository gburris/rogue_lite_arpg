mod empty_square;
mod hub;
mod temple;

use bevy::math::{Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

pub use empty_square::EmptySquare;
pub use hub::Hub;
pub use temple::Temple;

use crate::world::map::{MarkerType, map_data::MapData};

/// A trait for prefabricated map structures that can be placed in the game world
pub trait Prefab {
    /// Builds the prefab structure in the given map data
    ///
    /// # Arguments
    /// * `map_data` - The map data to build the structure in
    ///
    /// # Returns
    /// * `Option<Rect>` - The bounds of the built structure, if successful
    fn build(&self, map_data: &mut MapData) -> Option<Rect>;

    /// Gets the marker positions for this prefab
    ///
    /// # Arguments
    /// * `bounds` - The bounds of the built structure
    ///
    /// # Returns
    /// * `HashMap<MarkerType, Vec<Vec2>>` - A mapping of marker types to their positions
    fn get_markers(&self, bounds: &Rect) -> HashMap<MarkerType, Vec<Vec2>>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]

pub enum PrefabType {
    NPCHub,
    Temple,
    EmptySquare,
}

impl FromStr for PrefabType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NPCHub" => Ok(PrefabType::NPCHub),
            "Temple" => Ok(PrefabType::Temple),
            "EmptySquare" => Ok(PrefabType::EmptySquare),
            _ => Err(format!("Unknown prefab type: {}", s)),
        }
    }
}
