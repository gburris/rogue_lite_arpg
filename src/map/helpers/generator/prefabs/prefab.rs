use crate::map::{components::MarkerType, helpers::generator::MapData};
use bevy::math::{Rect, Vec2};
use std::collections::HashMap;

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
