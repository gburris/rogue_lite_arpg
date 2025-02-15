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
                    instance::spawn_instance_collisions_zones,
                    instance::spawn_background,
                    instance::spawn_instance_entities, //This is gonna mutate world size,
                    //Since it spawns a new map layout
                    instance::finish_create_instance,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(AppState::CreateHub),
                (
                    hub::generate_hub_layout,
                    instance::render_tilemap,
                    hub::spawn_hub_colliders,
                    hub::spawn_hub_entities,
                    hub::finish_create_hub,
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
