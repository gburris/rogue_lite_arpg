use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::map::{
    components::{StartingPortal, WarpZone},
    events::{DespawnAllPortals, DespawnAllTiles},
};

pub fn despawn_all_portals(
    _: Trigger<DespawnAllPortals>,
    mut commands: Commands,
    mut portal_query: Query<Entity, With<StartingPortal>>,
    mut warpzone_query: Query<Entity, With<WarpZone>>,
) {
    warn!("Depawning all portals");
    for entity in portal_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in warpzone_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn despawn_all_tiles(
    _: Trigger<DespawnAllTiles>,
    mut commands: Commands,
    mut tile_query: Query<Entity, With<TilemapId>>,
) {
    warn!("Depawning all tiles");
    for entity in tile_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
