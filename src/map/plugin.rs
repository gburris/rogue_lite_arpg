use bevy::prelude::*;

use crate::{
    labels::states::AppState,
    map::{
        resources::{CurrentZoneLevel, MapBounds, TileSize},
        systems::*,
    },
};

use super::WorldSpaceConfig;
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let tile_size: Vec2 = Vec2::new(16.0, 16.0);

        app.add_systems(Startup, instance::setup_instance_data)
            .add_systems(
                OnEnter(AppState::CreateInstance),
                (
                    instance::generate_instance_layout,
                    instance::render_instance_tilemap,
                    instance::spawn_instance_collisions_zones,
                    instance::spawn_instance_entities,
                    instance::finish_create_instance,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(AppState::CreateHub),
                (
                    hub::generate_hub_layout,
                    hub::render_hub_tiles,
                    hub::spawn_hub_colliders,
                    hub::spawn_hub_entities,
                    hub::finish_create_hub,
                )
                    .chain(),
            )
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
            })
            .add_observer(portal::on_portal_entered);
    }
}
