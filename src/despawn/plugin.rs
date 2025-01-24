use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    chests::components::Chest,
    combat::projectile::components::Projectile,
    despawn::{events::*, systems::*},
    enemy::Enemy,
    labels::sets::InGameSet,
    map::{
        components::{Portal, Wall},
        Water,
    },
    npc::NPC,
    player::Player,
    ui::{game_over_screen::RestartEvent, game_overlay::GameOverlay},
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (remove_expired_entities, despawn_on_zero_health).in_set(InGameSet::DespawnEntities),
        )
        .add_observer(despawn_all::<CleanupZone, Portal>)
        .add_observer(despawn_all::<CleanupZone, TilemapId>)
        .add_observer(despawn_all::<CleanupZone, Wall>)
        .add_observer(despawn_all::<CleanupZone, Water>)
        .add_observer(despawn_all::<CleanupZone, Chest>)
        .add_observer(despawn_all::<CleanupZone, Enemy>)
        .add_observer(despawn_all::<CleanupZone, Projectile>)
        .add_observer(despawn_all::<CleanupZone, NPC>)
        .add_observer(despawn_all::<RestartEvent, Player>)
        .add_observer(despawn_all::<RestartEvent, GameOverlay>)
        .add_observer(on_restart_event_cleanup_zone);
    }
}
