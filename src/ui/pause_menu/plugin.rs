use bevy::prelude::*;

use crate::{
    prelude::*,
    ui::{display_case, input},
};

use super::{button_interactions, inventory_menu, main_menu, stats_menu};
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Pause Related Systems
            .add_systems(
                Update,
                (
                    input::handle_ui_inputs,
                    display_case::update_scroll_position,
                )
                    .in_set(MainSystems::Menu),
            )
            // Main menu UI cisystems
            .add_systems(OnEnter(Menu::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                button_interactions::handle_menu_button_pressed
                    .run_if(in_state(Menu::MainMenu))
                    .in_set(MainSystems::Menu),
            )
            // Inventory menu systems
            .add_observer(display_case::on_display_case_updated)
            .add_systems(
                OnEnter(Menu::Inventory),
                inventory_menu::spawn_inventory_menu,
            )
            // Stats menu systems
            .add_systems(OnEnter(Menu::Stats), stats_menu::spawn_stats_menu);
    }
}
