use bevy::prelude::*;

use crate::{prelude::*, ui::{load_screen, start_screen, damage_overlay, game_over_screen}};

/// Plugin responsible for managing all UI-related systems and state transitions
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        //Loading screen
        app.add_systems(OnEnter(AppState::SpawnZone), load_screen::spawn)
            .add_systems(
                Update,
                (load_screen::animate_text).run_if(in_state(AppState::SpawnZone)),
            );

        // Start screen
        app.add_systems(OnEnter(AppState::StartScreen), start_screen::spawn)
            .add_systems(
                Update,
                (start_screen::button_system, start_screen::animate_text)
                    .run_if(in_state(AppState::StartScreen)),
            );

        // Heal and damage overlays
        app.add_observer(damage_overlay::on_damage_overlay_amount)
            .add_observer(damage_overlay::on_healing_overlay_amount);

        // Game over systems
        app.add_systems(OnEnter(AppState::GameOver), game_over_screen::spawn);
    }
}
