mod blueprint;
mod build;
mod placement;
mod spawn;
mod structures;
mod tiles;

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
// map_recipes.ron ──load──▶ RecipeCatalog { recipes, structures }
//                               │ MapRequest (Hub | Weighted | Named)
//                               ▼
// build_from_recipe(recipe, seed) ──▶ MapBlueprint { tiles, placements, visual_seed }
//   │ PlaceRule::{Fixed, Random, NearStructure, Edge}
//   │ + PlaceContext { reserved, occupied, structure_centers_by_name } (no-overlap arbitration)
//   ▼ OnEnter(SpawnMap), chained:
//   populate_map_tiles ─▶ "Tiles" child (bevy_ecs_tilemap, one tile_atlas.png,
//   │                        variant picked by (visual_seed, glyph, tile))
//   spawn_colliders    ─▶ merged Static rects over blocking runs (Wall/Water/DeadZone)
//   spawn_placements   ─▶ player transform │ SpawnEnemies/Npcs/Chests │ portal Exit
//   TileToWorld: tile ◄──► world conversions (single source of truth)
// ```
//
// Design notes:
// - build.rs owns determinism: one ChaCha8Rng(seed) threads through size rolls,
//   structure choice, and every PlaceRule. Same (recipe, seed) maps to the same blueprint.
// - MapBlueprint is renderer-agnostic data (TileGrid<TileKind> + placements).
//   All backend detail (TilemapBundle, atlas slots) stays inside spawn.rs.
// - Structures are glyph grids (: = transparent, letters = placements over floor);
//   builtin features (EmptySquare, Pond, ...) are picked by reserved kind name and
//   share footprint reservation. Portals sit one tile inside a solid wall border;
//   no doorway is carved.
// - validate() (single PlayerStart, at least one Exit, solid border, spawnable
//   disjoint placements) runs at generation time; startup additionally rebuilds
//   seeds 0..=8 per recipe as a smoke test.
