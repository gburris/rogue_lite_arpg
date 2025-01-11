use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    despawn::{events::CleanupCurrentWorldSpace, systems::*},
    enemy::Enemy,
    labels::sets::GamePlaySet,
    map::components::Portal,
    npc::NPC,
    projectile::Projectile,
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            remove_expired_entities.in_set(GamePlaySet::DespawnEntities),
        )
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Portal>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, TilemapId>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Enemy>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Projectile>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, NPC>);
    }
}
