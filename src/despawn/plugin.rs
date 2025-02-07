use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    chests::components::Chest,
    combat::projectile::components::Projectile,
    despawn::systems::*,
    enemy::Enemy,
    labels::{
        sets::InGameSet,
        states::{AppState, PausedState},
    },
    map::{components::Wall, events::CleanupZone, portal::Portal, Water},
    npc::NPC,
    player::Player,
    ui::{
        game_over_screen::RestartEvent,
        npc::stats_shop::StatsMenu,
        pause_menu::{inventory_menu::InventoryMenu, main_menu::MainMenu, pause::PauseBackground},
        player_overlay::GameOverlay,
    },
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_expired_entities).in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnExit(AppState::Paused), despawn_single::<PauseBackground>)
        .add_systems(
            OnExit(PausedState::Inventory),
            despawn_single::<InventoryMenu>,
        )
        .add_systems(OnExit(PausedState::Stats), despawn_single::<StatsMenu>)
        .add_systems(OnExit(PausedState::MainMenu), despawn_single::<MainMenu>)
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
