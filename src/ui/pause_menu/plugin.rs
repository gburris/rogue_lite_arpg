use bevy::prelude::*;

use crate::{
    labels::{
        sets::MainSet,
        states::{AppState, PausedState},
    },
    ui::input,
    ui::time_control,
};

use super::{
    button_interactions, equipment_menu, inventory_menu, main_menu, pause_screen, stats_menu,
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
                button_interactions::handle_menu_button_pressed
                    .run_if(in_state(PausedState::MainMenu))
                    .in_set(MainSet::Menu),
            )
            .add_systems(OnExit(PausedState::MainMenu), main_menu::despawn_main_menu)
            // Equipment menu systems
            .add_observer(button_interactions::handle_equipment_click)
            .add_observer(equipment_menu::handle_equipment_update)
            .add_systems(
                OnEnter(PausedState::Equipment),
                equipment_menu::spawn_equipment_menu,
            )
            .add_systems(
                Update,
                button_interactions::handle_equipment_interactions
                    .run_if(in_state(PausedState::Equipment)),
            )
            .add_systems(
                OnExit(PausedState::Equipment),
                equipment_menu::despawn_equipment_menu,
            )
            // Inventory menu systems
            .add_observer(button_interactions::handle_inventory_click)
            .add_observer(inventory_menu::handle_inventory_update)
            .add_systems(
                OnEnter(PausedState::Inventory),
                inventory_menu::spawn_inventory_menu,
            )
            .add_systems(
                Update,
                button_interactions::handle_inventory_interactions
                    .run_if(in_state(PausedState::Inventory)),
            )
            .add_systems(
                OnExit(PausedState::Inventory),
                inventory_menu::despawn_inventory_menu,
            )
            // Stats menu systems
            .add_systems(OnEnter(PausedState::Stats), stats_menu::spawn_stats_menu)
            .add_systems(OnExit(PausedState::Stats), stats_menu::despawn_stats_menu);
    }
}
