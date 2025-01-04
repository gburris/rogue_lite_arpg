use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum GameState {
    #[default]
    AssetLoading,
    SpawnPlayer,
    CreateOverworld,
    CreateZone,
    PlayingOnOverWorld, //I also create UI here, I wonder if I should split this into two states
    PlayingInZone,
    Paused,
    GameOver,
}
