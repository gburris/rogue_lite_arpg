use bevy::prelude::*;

use crate::labels::{
    sets::{InGameSet, MainSet},
    states::AppState,
};

use super::{
    game_over_screen::{self, handle_restart_button},
    game_overlay,
};

/// Plugin responsible for managing all UI-related systems and state transitions
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // Game UI systems
        app
            // Core game overlay (HUD)
            .add_systems(OnEnter(AppState::SpawnPlayer), game_overlay::spawn)
            .add_systems(Update, game_overlay::update.in_set(InGameSet::HudOverlay))
            // Game over systems
            .add_systems(OnEnter(AppState::GameOver), game_over_screen::create)
            .add_systems(
                OnExit(AppState::GameOver),
                game_over_screen::despawn_game_over_screen,
            )
            .add_systems(
                Update,
                handle_restart_button
                    .run_if(in_state(AppState::GameOver))
                    .in_set(MainSet::Menu),
            );
    }
}
