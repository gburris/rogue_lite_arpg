use bevy::prelude::*;

use crate::{
    labels::{sets::InGameSet, states::AppState},
    map::{portal, resources::CurrentZoneLevel, systems::*},
};

use super::{portal::on_mapper_spawned, WorldSpaceConfig};
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, instance::setup_instance_data)
            .add_systems(
                OnEnter(AppState::CreateInstance),
                (
                    instance::render_tilemap,
                    instance::spawn_zone_colliders,
                    instance::spawn_background,
                    instance::spawn_instance_entities, //This is gonna mutate world size,
                    //Since it spawns a new map layout
                    instance::finish_create_zone,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(AppState::CreateHub),
                (
                    hub::insert_hub_layout,
                    instance::render_tilemap,
                    instance::spawn_zone_colliders,
                    hub::spawn_hub_entities,
                    instance::finish_create_zone,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (portal::handle_portal_collisions).in_set(InGameSet::Collision),
            )
            .insert_resource(WorldSpaceConfig::default())
            .insert_resource(CurrentZoneLevel(0))
            .add_observer(portal::on_portal_entered)
            .add_observer(on_mapper_spawned);
    }
}
