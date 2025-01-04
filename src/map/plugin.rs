use crate::{
    labels::{sets::GamePlaySet, states::GameState},
    map::{
        events::{StartRunEvent, WarpZoneEnterEvent},
        systems::*,
    },
    resources::{MapBounds, TileSize},
};

use bevy::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let tile_size_x = 16.0;
        let tile_size_y = 16.0;
        app.add_systems(
            OnEnter(GameState::CreateZone),
            (generate_tilemap, warpzone_setup).chain(),
        )
        .add_systems(
            OnEnter(GameState::CreateOverworld),
            (generate_tilemap_for_overworld, starting_portal_setup).chain(),
        )
        .add_systems(
            Update,
            (handle_warpzone_enter, enter_start_portal).in_set(GamePlaySet::DespawnEntities),
        )
        .add_event::<WarpZoneEnterEvent>()
        .add_event::<StartRunEvent>()
        .insert_resource(TileSize { x: 16.0, y: 16.0 })
        .insert_resource(MapBounds {
            min_x: -100.0 * tile_size_x,
            min_y: -100.0 * tile_size_y,
            max_x: 100.0 * tile_size_x,
            max_y: 100.0 * tile_size_y,
        });
    }
}
