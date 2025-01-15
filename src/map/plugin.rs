use bevy::prelude::*;

use crate::{
    labels::{
        sets::InGameSet,
        states::{AppState, InGameState},
    },
    map::{
        events::{StartRunEvent, WarpZoneEnterEvent},
        resources::{CurrentZoneLevel, MapBounds, TileSize},
        systems::*,
    },
};
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let tile_size_x = 16.0;
        let tile_size_y = 16.0;
        app.add_systems(
            OnEnter(AppState::CreateZone),
            (generate_tilemap, warpzone_setup).chain(),
        )
        .add_systems(
            OnEnter(AppState::CreateOverworld),
            (generate_tilemap_for_overworld, starting_portal_setup).chain(),
        )
        .add_systems(
            Update,
            (
                handle_warpzone_enter.run_if(in_state(InGameState::Run)),
                enter_start_portal.run_if(in_state(InGameState::BeforeRun)),
            )
                .in_set(InGameSet::Simulation),
        )
        .add_event::<WarpZoneEnterEvent>()
        .add_event::<StartRunEvent>()
        .insert_resource(TileSize {
            x: tile_size_x,
            y: tile_size_y,
        })
        .insert_resource(MapBounds {
            min_x: -100.0 * tile_size_x,
            min_y: -100.0 * tile_size_y,
            max_x: 100.0 * tile_size_x,
            max_y: 100.0 * tile_size_y,
        })
        .insert_resource(CurrentZoneLevel(0));
    }
}
