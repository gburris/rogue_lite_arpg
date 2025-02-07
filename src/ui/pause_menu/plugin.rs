use bevy::prelude::*;

use crate::{
    labels::{
        sets::MainSet,
        states::{AppState, PausedState},
    },
    ui::{input, time_control},
};

use super::{
    button_interactions,
    equipment_menu::{self},
    inventory_menu, main_menu, pause, stats_menu,
};
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Pause Related Systems
            .add_observer(input::on_pause_input)
            .add_systems(Update, input::handle_ui_inputs.in_set(MainSet::Menu))
            .add_systems(
                OnEnter(AppState::Paused),
                (time_control::pause_game, pause::spawn_pause_background),
            )
            .add_systems(OnExit(AppState::Paused), time_control::resume_game)
            // Main menu UI cisystems
            .add_systems(OnEnter(PausedState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                button_interactions::handle_menu_button_pressed
                    .run_if(in_state(PausedState::MainMenu))
                    .in_set(MainSet::Menu),
            )
            // Inventory menu systems
            .add_observer(inventory_menu::handle_inventory_update)
            .add_systems(
                OnEnter(PausedState::Inventory),
                inventory_menu::spawn_inventory_menu,
            )
            // Stats menu systems
            .add_systems(OnEnter(PausedState::Stats), stats_menu::spawn_stats_menu);
    }
}
