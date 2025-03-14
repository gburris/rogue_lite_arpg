use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateSet;

use crate::labels::{
    sets::{InGameSet, MainSet},
    states::AppState,
};

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MainSet::InGame.run_if(in_state(AppState::Playing)),
                MainSet::Menu.run_if(in_state(AppState::Paused)),
                MainSet::Shared,
            )
                .chain()
                .after(LoadingStateSet(AppState::AssetLoading)), // appease the system ordering gods
        )
        // Configuring the ordering of our gameplay loop using these main sets:
        // Despawn Entitites -> Handle Input -> Simulation -> Update HUD / overlay UI -> Physics -> Collision
        .configure_sets(
            Update,
            (
                // Since 0.13, apply_deferred is automatically applied when a command is run in a system
                // This ensures entities are always despawned before this frames simulation runs
                InGameSet::DespawnEntities,
                InGameSet::PlayerInput,
                InGameSet::Simulation,
                InGameSet::HudOverlay,
                InGameSet::Physics,
                InGameSet::Collision,
            )
                .chain()
                .in_set(MainSet::InGame),
        );
    }
}
