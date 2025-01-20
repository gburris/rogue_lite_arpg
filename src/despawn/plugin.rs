use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    combat::projectile::components::Projectile,
    despawn::{events::CleanupCurrentWorldSpace, systems::*},
    enemy::Enemy,
    labels::sets::InGameSet,
    map::{components::Portal, systems::tilemap_generation::Wall},
    npc::NPC,
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (remove_expired_entities, despawn_on_zero_health).in_set(InGameSet::DespawnEntities),
        )
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Portal>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, TilemapId>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Wall>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Enemy>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, Projectile>)
        .add_observer(despawn_all::<CleanupCurrentWorldSpace, NPC>);
    }
}
