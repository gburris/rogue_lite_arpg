use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum AppState {
    #[default]
    StartScreen,
    AssetLoading,
    CreateHub,
    SpawnPlayer, // Also creates player overlay UI
    CreateInstance,
    // Represents time in game where all game-play systems are actually running
    // If you want to distinguish between different phases within "Playing" make a new state!
    // Ex. MapRegionState
    Playing,
    Paused,
    GameOver,
}

#[derive(SubStates, Eq, Default, Hash, Clone, Copy, Debug, PartialEq)]
#[source(AppState = AppState::Paused)]
pub enum PausedState {
    #[default]
    MainMenu,
    Inventory,
    Equipment,
    Stats,
}
