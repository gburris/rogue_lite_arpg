use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateSet;

use crate::labels::{
    sets::{InGameSystems, MainSystems},
    states::AppState,
};

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MainSystems::InGame.run_if(in_state(AppState::Playing)),
                MainSystems::Menu.run_if(in_state(AppState::Paused)),
                MainSystems::Shared,
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
                InGameSystems::DespawnEntities,
                InGameSystems::PlayerInput,
                InGameSystems::Simulation,
                InGameSystems::Collision,
                InGameSystems::Vfx,
                InGameSystems::HudOverlay,
            )
                .chain()
                .in_set(MainSystems::InGame),
        )
        .configure_sets(
            FixedUpdate,
            MainSystems::InGame.run_if(in_state(AppState::Playing)),
        );
    }
}
