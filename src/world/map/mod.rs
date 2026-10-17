mod blueprint;
mod build;
mod prefabs;
mod spawn;
#[allow(dead_code)]
mod structure;
mod tiles;

use ::bevy::prelude::*;

pub mod prelude {
    pub use super::TILE_SIZE;
    pub use super::blueprint::*;
    pub use super::build::*;
    pub use super::prefabs::*;
    pub use super::spawn::*;
    pub use super::tiles::*;
}

pub const TILE_SIZE: f32 = 32.0;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((build::plugin, spawn::plugin));
}

// High level build -> blueprint -> spawn
// build can be: from ron, from ascii, from ldtk, from in-code procedural
// blueprint is defined interface
// spawn today is bevy_ecs_tilemap but could later be built in bevy tilemap, custom, or something else entirely

// most of complexity is going from build -> blueprint
// what if we want randomization? what if we want hand crafted level? what if we want a static building within random world

// ELEMENTS OF BUILD

// Structure: Predefined tiles in a specific layout and optionally placements within/around said tiles
// Examples: hub town, temple

// Feature: Tiles taking a certain shape and behaving a certain way, but procedurally generated
// Examples: walled off square of random size, walls, ponds, other random tile hazards like traps of various sizes

// MapParameters: Things that can be tuned per-map, can be Random or Static
// Examples: number & location of enemies, chests, npcs, size of map

// Layout: Describes list of MapParameters to make a MapBlueprint

// HubTown Layout:
// size: (100, 100)
// structures: [Static(HubTown, (0,0))]
// npcs: Static([(Shopkeeper, (10, 5)), (Blacksmith, (-10,5)), ...])
//
// SwampDungeon Layout:
// size: (200, 400) // make non-rectangle maps later
// structure: [Static(Temple, location=(10, 20))]
// features: [Random(WallSquare, amount[3,8]), ]

// Element of every blueprint: build of tiles, has a defined size, has walls around the border, has an initial floor type (can be changed obv)
