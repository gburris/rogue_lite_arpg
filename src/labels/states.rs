use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum GameState {
    #[default]
    AssetLoading,
    CreateOverworld,
    SpawnPlayer, // Also creates UI
    CreateZone,
    // Represents time in game where all game-play systems are actually running
    // If you want to distinguish between different phases within "Playing" make a new state!
    // Ex. MapRegionState
    Playing,
    CleanupZone,
    Paused,
    GameOver,
}
