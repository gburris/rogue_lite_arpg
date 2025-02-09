use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    chests::components::Chest,
    combat::projectile::components::Projectile,
    despawn::systems::*,
    enemy::Enemy,
    labels::sets::InGameSet,
    map::{components::Wall, events::CleanupZone, portal::Portal, Water},
    npc::NPC,
    player::Player,
    ui::{game_over_screen::RestartEvent, player_overlay::GameOverlay},
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_expired_entities).in_set(InGameSet::DespawnEntities),
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
        .add_observer(despawn_all::<RestartEvent, GameOverlay>);
    }
}
