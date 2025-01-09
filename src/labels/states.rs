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
    Paused,
    GameOver,
}

// A sub state for when we are in playing.
// This is useful for when we want to distinguish between different phases within "Playing"
#[derive(States, Eq, Default, Hash, Clone, Debug, PartialEq)]
pub enum PlayingState {
    #[default]
    BeforeRun, // Overworld State
    Run, // Post entering the start portal state
}
