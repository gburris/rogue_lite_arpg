use bevy::prelude::*;
use instance::{
    finish_create_instance, generate_instance_layout, render_instance_tilemap,
    spawn_instance_collisions_zones, spawn_instance_entities,
};

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

use super::WorldSpaceConfig;
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let tile_size: Vec2 = Vec2::new(16.0, 16.0);
        app.add_systems(
            OnEnter(AppState::CreateInstance),
            (
                generate_instance_layout,
                render_instance_tilemap,
                spawn_instance_collisions_zones,
                spawn_instance_entities,
                finish_create_instance,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(AppState::CreateOverworld),
            (generate_tilemap_for_overworld, starting_portal_setup).chain(),
        )
        // .add_systems(
        //     OnEnter(AppState::CreateOverworld),
        //     (
        //         generate_overworld_layout,
        //         render_overworld_tilemap,
        //         spawn_overworld_collisions_zones,
        //         spawn_overworld_entities,
        //         finish_create_overworld,
        //     )
        //         .chain(),
        // )
        //.add_systems(OnExit(InGameState::Run), (despawn_walls).chain())
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
        .insert_resource(WorldSpaceConfig::default())
        .insert_resource(CurrentZoneLevel(0))
        .insert_resource(TileSize {
            x: tile_size.x,
            y: tile_size.y,
        })
        .insert_resource(MapBounds {
            min_x: -100.0 * tile_size.x,
            min_y: -100.0 * tile_size.y,
            max_x: 100.0 * tile_size.x,
            max_y: 100.0 * tile_size.y,
        });
    }
}
