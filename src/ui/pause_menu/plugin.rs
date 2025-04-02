use bevy::prelude::*;

use crate::{
    configuration::time_control,
    labels::{
        sets::MainSet,
        states::{AppState, PausedState},
    },
    ui::{display_case, input},
};

use super::{button_interactions, inventory_menu, main_menu, stats_menu};
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(input::on_pause_input_event)
            // Pause Related Systems
            .add_systems(
                Update,
                (display_case::update_scroll_position,).in_set(MainSet::Menu),
            )
            .add_systems(OnEnter(AppState::Paused), (time_control::pause_game,))
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
            .add_observer(display_case::on_display_case_updated)
            .add_systems(
                OnEnter(PausedState::Inventory),
                inventory_menu::spawn_inventory_menu,
            )
            // Stats menu systems
            .add_systems(OnEnter(PausedState::Stats), stats_menu::spawn_stats_menu);
    }
}
