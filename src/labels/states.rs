use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    StartScreen,
    AssetLoading,
    CreateHub,
    SpawnPlayer, // Also creates player overlay UI
    SpawnZone,   //Used to 1. put a load screen on 2. Spawn everything in ur zone
    Playing,
    Paused,
    GameOver,
}

#[derive(SubStates, Eq, Default, Hash, Clone, Copy, Debug, PartialEq)]
#[source(AppState = AppState::Paused)]
#[states(scoped_entities)]
pub enum PausedState {
    #[default]
    MainMenu,
    Inventory,
    StatsShop,
    ItemsShop,
    Stats,
}

#[derive(SubStates, Eq, Hash, Default, Clone, Copy, Debug, PartialEq)]
#[source(AppState = AppState::Playing)]
pub enum PlayingState {
    #[default]
    Playing,
    Death,
}
