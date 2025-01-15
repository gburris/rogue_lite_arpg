use bevy::prelude::*;

use crate::labels::{
    sets::GamePlaySet,
    states::{GameState, PausedState},
};

use super::{
    equipment_menu,
    game_over_screen::{self, handle_restart_button},
    game_overlay,
    pause_menu::{self, handle_equipment_button_pressed, on_pause_input, ui_inputs},
    time_control,
};

/// Plugin responsible for managing all UI-related systems and state transitions
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // Game UI systems
        app.configure_sets(Update, GamePlaySet::UI.after(GamePlaySet::Simulation))
            // Core game overlay (HUD)
            .add_systems(OnEnter(GameState::SpawnPlayer), game_overlay::spawn)
            .add_systems(Update, game_overlay::update.in_set(GamePlaySet::UI))
            // Pause Related Systems
            .add_observer(on_pause_input)
            .add_systems(Update, ui_inputs)
            .add_systems(
                OnEnter(GameState::Paused(PausedState::Enter)),
                (
                    time_control::pause_game,
                    pause_menu::spawn_pause_screen,
                    pause_menu::set_default_menu_state,
                ),
            )
            .add_systems(
                OnEnter(GameState::Paused(PausedState::Exit)),
                (time_control::resume_game, pause_menu::despawn_pause_screen),
            )
            // Main menu UI cisystems
            .add_systems(
                OnEnter(GameState::Paused(PausedState::MainMenu)),
                pause_menu::spawn_main_menu,
            )
            .add_systems(
                Update,
                handle_equipment_button_pressed
                    .run_if(in_state(GameState::Paused(PausedState::MainMenu))),
            )
            .add_systems(
                OnExit(GameState::Paused(PausedState::MainMenu)),
                pause_menu::despawn_main_menu,
            )
            // Equipment menu systems
            .add_systems(
                OnEnter(GameState::Paused(PausedState::Equipment)),
                equipment_menu::spawn_equipment_menu,
            )
            .add_systems(
                OnExit(GameState::Paused(PausedState::Equipment)),
                equipment_menu::despawn_equipment_menu,
            )
            // Game over systems
            .add_systems(OnEnter(GameState::GameOver), game_over_screen::create)
            .add_systems(
                OnExit(GameState::GameOver),
                game_over_screen::despawn_game_over_screen,
            )
            // Global UI interaction systems
            .add_systems(
                Update,
                (handle_restart_button.run_if(in_state(GameState::GameOver)),),
            );
    }
}
