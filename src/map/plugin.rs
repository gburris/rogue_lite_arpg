use bevy::prelude::*;

use crate::{
    labels::{sets::InGameSystems, states::AppState},
    map::{components::WorldSpaceConfig, portal, systems::*},
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_instance_data)
            .add_systems(
                OnEnter(AppState::SpawnZone),
                (
                    zone::despawn_previous_zone,
                    zone::spawn_zone_tilemap,
                    zone::spawn_zone_colliders,
                    //zone::spawn_background,
                    zone::spawn_zone_entities,
                    zone::finish_create_zone,
                )
                    .chain(),
            )
            .add_systems(OnEnter(AppState::CreateHub), (insert_hub_layout,).chain())
            .add_systems(
                Update,
                (portal::handle_portal_collisions).in_set(InGameSystems::Collision),
            )
            .insert_resource(WorldSpaceConfig::default())
            .add_observer(portal::on_portal_entered);
    }
}
