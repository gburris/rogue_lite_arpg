use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum GameState {
    #[default]
    AssetLoading,
    SpawnPlayer,
    CreateLevel,
    Playing,
    Paused,
    GameOver,
}
