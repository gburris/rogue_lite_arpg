mod blueprint;
mod build;
mod placement;
mod recipe;
mod spawn;
mod structures;
mod tiles;
mod tiling;

use ::bevy::prelude::*;

/// Convenient exports for map recipe, blueprint, structure, tile, and spawn APIs.
pub mod prelude {
    pub use super::blueprint::*;
    pub use super::build::*;
    pub use super::placement::*;
    pub use super::spawn::*;
    pub use super::structures::*;
    pub use super::tiles::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((build::plugin, spawn::plugin));
}

// Map pipeline: recipe -> blueprint -> spawned world.
//
// ```text
// map_recipes.ron ──load (recipe.rs)──▶ RecipeCatalog { recipes, structures }
//                               │ MapRequest (Hub | Weighted | Named)
//                               ▼
// build_from_recipe(recipe, seed) ──▶ MapBlueprint { tiles, placements, recipe, seed }
//   │ PlaceRule::{Fixed, Random, NearStructure, Edge}
//   │ + PlaceContext { reserved, structure_centers_by_name, points, keep_clear }
//   ▼ OnEnter(SpawnMap), chained:
//   populate_map_tiles ─▶ "Tiles" child (bevy_ecs_tilemap, one tile_atlas.png,
//   │                        texture picked by (seed, tile, neighbours):
//   │                        centre variants, wall top/face pieces, bank edges)
//   spawn_colliders    ─▶ merged Static rects over blocking runs (Wall/Water/DeadZone)
//   spawn_placements   ─▶ player transform │ SpawnEnemies/Npcs/Chests │ portal Exit
//   TileToWorld: tile ◄──► world conversions (single source of truth)
// ```
//
// Design notes:
// - build.rs owns determinism: one ChaCha8Rng(seed) threads through size rolls,
//   structure choice, and every PlaceRule. Same (recipe, seed) maps to the same blueprint,
//   and the blueprint keeps both, so MapRequest::Named { recipe, seed } replays it.
//   The weighted recipe pick draws from the root MapRng, never from the build seed.
// - Build order: structures, fixed enemies, player start (kept PLAYER_START_CLEARANCE
//   tiles from enemies), exits, random enemies, chests. Separation is measured against
//   every point placed so far, not just the current batch.
// - Exits and the player start each come from exactly one source: structure glyphs or
//   a recipe rule.
// - MapBlueprint is renderer-agnostic data (TileGrid<TileKind> + placements).
//   All backend detail (TilemapBundle, atlas slots) stays inside spawn.rs.
// - Structures are glyph grids (: = transparent, letters = placements over floor);
//   builtin features (EmptySquare, Pond, ...) are picked by reserved kind name and
//   share footprint reservation. Portals sit one tile inside a solid wall border;
//   no doorway is carved.
// - validate() (single PlayerStart, at least one Exit, solid border, spawnable
//   disjoint placements) runs at generation time; startup additionally rebuilds
//   seeds 0..=8 per recipe as a smoke test, and `cargo test` builds seeds 0..64.
// - Dev builds: editing map_recipes.ron rebuilds the current map in place; F6 cycles
//   through every recipe in file order.
