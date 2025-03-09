use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    combat::projectile::components::Projectile,
    configuration::time_control::RestartEvent,
    despawn::systems::*,
    enemy::Enemy,
    items::Lootable,
    labels::{sets::InGameSet, states::PausedState},
    map::{
        components::Wall, portal::Portal, systems::zone::ZoneBackground, Chest, CleanupZone, Water,
    },
    npc::NPC,
    player::Player,
    ui::{InventoryMenu, MainMenu, PlayerOverlay, StatShopMenu, StatsMenu},
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_expired_entities).in_set(InGameSet::DespawnEntities),
        )
        .add_systems(
            OnExit(PausedState::Inventory),
            despawn_single::<InventoryMenu>,
        )
        .add_systems(OnExit(PausedState::Stats), despawn_single::<StatsMenu>)
        .add_systems(
            OnExit(PausedState::StatsShop),
            despawn_single::<StatShopMenu>,
        )
        .add_systems(OnExit(PausedState::MainMenu), despawn_single::<MainMenu>)
        .add_observer(despawn_all::<CleanupZone, Portal>)
        .add_observer(despawn_all::<CleanupZone, TilemapId>)
        .add_observer(despawn_all::<CleanupZone, Wall>)
        .add_observer(despawn_all::<CleanupZone, Water>)
        .add_observer(despawn_all::<CleanupZone, ZoneBackground>)
        .add_observer(despawn_all::<CleanupZone, Lootable>)
        .add_observer(despawn_all::<CleanupZone, Chest>)
        .add_observer(despawn_all::<CleanupZone, Enemy>)
        .add_observer(despawn_all::<CleanupZone, Projectile>)
        .add_observer(despawn_all::<CleanupZone, NPC>)
        .add_observer(despawn_all::<RestartEvent, Player>)
        .add_observer(despawn_all::<RestartEvent, PlayerOverlay>);
    }
}
