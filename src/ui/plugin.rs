use bevy::prelude::*;

use crate::labels::{
    sets::{InGameSet, MainSet},
    states::{AppState, PausedState},
};

use super::{
    equipment_menu,
    game_over_screen::{self, handle_restart_button},
    game_overlay,
    pause_menu::{self, handle_equipment_button_pressed, handle_ui_inputs, on_pause_input},
    time_control,
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
            // Pause Related Systems
            .add_observer(on_pause_input)
            .add_systems(Update, handle_ui_inputs.in_set(MainSet::Menu))
            .add_systems(
                OnEnter(AppState::Paused),
                (time_control::pause_game, pause_menu::spawn_pause_screen),
            )
            .add_systems(
                OnExit(AppState::Paused),
                (time_control::resume_game, pause_menu::despawn_pause_screen),
            )
            // Main menu UI cisystems
            .add_systems(OnEnter(PausedState::MainMenu), pause_menu::spawn_main_menu)
            .add_systems(
                Update,
                handle_equipment_button_pressed
                    .run_if(in_state(PausedState::MainMenu))
                    .in_set(MainSet::Menu),
            )
            .add_systems(OnExit(PausedState::MainMenu), pause_menu::despawn_main_menu)
            // Equipment menu systems
            .add_systems(
                OnEnter(PausedState::Equipment),
                equipment_menu::spawn_equipment_menu,
            )
            .add_systems(
                OnExit(PausedState::Equipment),
                equipment_menu::despawn_equipment_menu,
            )
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
