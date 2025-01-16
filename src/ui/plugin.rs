use bevy::prelude::*;

use crate::labels::{
    sets::{InGameSet, MainSet},
    states::{AppState, PausedState},
};

use super::{
    equipment_menu,
    game_over_screen::{self, handle_restart_button},
    game_overlay,
    input::{handle_ui_inputs, on_pause_input},
    inventory_menu,
    main_menu::{self, handle_menu_button_pressed},
    pause_screen, stats_menu, time_control,
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
                (time_control::pause_game, pause_screen::spawn_pause_screen),
            )
            .add_systems(
                OnExit(AppState::Paused),
                (
                    time_control::resume_game,
                    pause_screen::despawn_pause_screen,
                ),
            )
            // Main menu UI cisystems
            .add_systems(OnEnter(PausedState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                handle_menu_button_pressed
                    .run_if(in_state(PausedState::MainMenu))
                    .in_set(MainSet::Menu),
            )
            .add_systems(OnExit(PausedState::MainMenu), main_menu::despawn_main_menu)
            // Equipment menu systems
            .add_systems(
                OnEnter(PausedState::Equipment),
                equipment_menu::spawn_equipment_menu,
            )
            .add_systems(
                OnExit(PausedState::Equipment),
                equipment_menu::despawn_equipment_menu,
            )
            // Inventory menu systems
            .add_systems(
                OnEnter(PausedState::Inventory),
                inventory_menu::spawn_inventory_menu,
            )
            .add_systems(
                OnExit(PausedState::Inventory),
                inventory_menu::despawn_inventory_menu,
            )
            // Stats menu systems
            .add_systems(OnEnter(PausedState::Stats), stats_menu::spawn_stats_menu)
            .add_systems(OnExit(PausedState::Stats), stats_menu::despawn_stats_menu)
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
