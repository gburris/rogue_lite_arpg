use crate::{
    labels::{sets::GamePlaySet, states::GameState},
    resources::{MapBounds, TileSize},
};

use bevy::prelude::*;

use super::{generate_tilemap, handle_warpzone_enter, warpzone_setup, WarpZoneEnterEvent};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let tile_size_x = 16.0;
        let tile_size_y = 16.0;
        app.add_systems(
            OnEnter(GameState::CreateLevel),
            (generate_tilemap, warpzone_setup).chain(),
        )
        .add_systems(
            Update,
            handle_warpzone_enter.in_set(GamePlaySet::Simulation),
        )
        .add_event::<WarpZoneEnterEvent>()
        .insert_resource(TileSize { x: 16.0, y: 16.0 })
        .insert_resource(MapBounds {
            min_x: -100.0 * tile_size_x,
            min_y: -100.0 * tile_size_y,
            max_x: 100.0 * tile_size_x,
            max_y: 100.0 * tile_size_y,
        });
    }
}
