use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateSet;

use crate::labels::{
    sets::{GamePlaySet, MainSet},
    states::GameState,
};

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            MainSet::GamePlay.run_if(
                in_state(GameState::PlayingInZone).or(in_state(GameState::PlayingOnOverWorld)),
            ),
        )
        // Configuring the ordering of our gameplay loop using these main sets:
        // Despawn Entitites -> Handle Input -> Simulation -> Physics -> Collision
        .configure_sets(
            Update,
            (
                // Since 0.13, apply_deferred is automatically applied when a command is run in a system
                // This ensures entities are always despawned before this frames simulation runs
                GamePlaySet::DespawnEntities.after(LoadingStateSet(GameState::AssetLoading)), // appease the system ordering gods,
                GamePlaySet::PlayerInput,
                GamePlaySet::Simulation,
                GamePlaySet::Collision,
            )
                .chain()
                .in_set(MainSet::GamePlay),
        );
    }
}
