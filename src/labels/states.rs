use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum GameState {
    #[default]
    AssetLoading,
    SpawnPlayer,
    CreateLevel, //I also create UI here, I wonder if I should split this into two states
    Playing,
    Paused,
    GameOver,
}
