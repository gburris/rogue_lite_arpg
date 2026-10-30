//! Map pipeline: recipe -> blueprint -> spawned world.
//!
//! ```text
//! map_recipes.ron ──load (recipe.rs)──▶ RecipeCatalog { recipes, structures }
//!                               │ MapRequest (Hub | Weighted | Named)
//!                               ▼
//! build_from_recipe(recipe, seed) ──▶ MapBlueprint { tiles, placements, recipe, seed }
//!   │ PlaceRule::{Fixed, Random, NearStructure, Edge}
//!   │ + PlaceContext { reserved, structure_centers_by_name, points, keep_clear }
//!   ▼ OnEnter(SpawnMap), chained:
//!   populate_map_tiles ─▶ "Tiles" child (bevy_ecs_tilemap, one tile_atlas.png,
//!   │                        texture picked by (seed, tile, neighbours):
//!   │                        centre variants, wall top/face pieces, bank edges)
//!   spawn_colliders    ─▶ merged Static rects over blocking runs (Wall/Water/DeadZone)
//!   spawn_placements   ─▶ player transform │ SpawnEnemies/Npcs/Chests │ portal Exit
//!   TileToWorld: tile ◄──► world conversions (single source of truth)
//! ```
//!
//! Design notes:
//! - `build.rs` owns determinism: the same (recipe, seed) maps to the same blueprint, and
//!   the blueprint keeps both, so `MapRequest::Named { recipe, seed }` replays it.
//! - Recipes list everything they place in `spawns` (what, count, at, separation);
//!   structures and spawns pick their location from the same `Location` options.
//! - Build order: size, structures, fixed-anchor spawns, player start (kept the recipe's
//!   `start_clearance` from enemies), exits, enemies, chests, NPCs (see `Stage`).
//! - `MapBlueprint` is renderer-agnostic data (`TileGrid<TileKind>` + placements).
//!   All backend detail (`TilemapBundle`, atlas slots) stays inside `spawn.rs`.
//! - Structures are glyph grids (: = transparent, letters = placements over floor);
//!   builtin features (`EmptySquare`, `Pond`, ...) are picked by reserved kind name and
//!   share footprint reservation. Portals sit one tile inside a solid wall border;
//!   no doorway is carved.
//! - `validate()` (single `PlayerStart`, at least one `Exit`, solid border, spawnable
//!   disjoint placements) runs at generation time; startup additionally rebuilds
//!   seeds 0..=8 per recipe as a smoke test.
//! - Dev builds: editing `map_recipes.ron` rebuilds the current map in place; F1 opens the
//!   `AiTestArena` arena; F6 cycles through every recipe in file order.

mod blueprint;
mod build;
mod placement;
mod recipe;
mod spawn;
mod structures;
mod tiles;
mod tiling;

use ::bevy::prelude::*;

/// Map exports other features use: map requests, placement anchors, and tile-to-world lookup.
pub mod prelude {
    pub use super::build::{MapRequest, NextMap};
    pub use super::placement::{Anchor, place};
    pub use super::spawn::{TILE_SIZE, TileToWorld};
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((build::plugin, spawn::plugin));
}
