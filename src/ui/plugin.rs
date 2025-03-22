use bevy::prelude::*;

use crate::{
    labels::{sets::InGameSet, states::AppState},
    ui::*,
};

use super::{loading::plugin::LoadingUIPlugin, npc::plugin::NPCPauseScreensPlugin};

/// Plugin responsible for managing all UI-related systems and state transitions
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // Game UI systems
        app
            //Other UI Plugins here
            .add_plugins(PauseMenuPlugin)
            .add_plugins(NPCPauseScreensPlugin)
            .add_plugins(LoadingUIPlugin)
            //Start screen
            .add_systems(
                OnEnter(AppState::StartScreen),
                start_screen::spawn_start_screen,
            )
            .add_systems(
                OnExit(AppState::StartScreen),
                start_screen::despawn_start_screen,
            )
            .add_systems(
                Update,
                (start_screen::button_system, start_screen::animate_text)
                    .run_if(in_state(AppState::StartScreen)),
            )
            // Core game overlay (HUD)
            .add_systems(OnEnter(AppState::SpawnPlayer), player_overlay::spawn)
            .add_systems(
                Update,
                (
                    player_overlay::update_exp_bar,
                    player_overlay::update_action_bar,
                    player_overlay::update_cooldowns,
                    (
                        player_overlay::update_mana_bar,
                        player_overlay::update_lost_mana_bar,
                    )
                        .chain(),
                    (
                        player_overlay::update_health_bar,
                        player_overlay::update_lost_health_bar,
                    )
                        .chain(),
                )
                    .in_set(InGameSet::HudOverlay),
            )
            .add_observer(damage_overlay::on_damage_overlay_amount)
            .add_observer(damage_overlay::on_healing_overlay_amount)
            // Game over systems
            .add_systems(OnEnter(AppState::GameOver), game_over_screen::create)
            .add_systems(
                OnExit(AppState::GameOver),
                game_over_screen::despawn_game_over_screen,
            )
            .add_observer(game_over_screen::on_restart_event_cleanup_zone)
            .add_observer(player_overlay::on_equipment_used)
            .add_observer(player_overlay::on_equipment_use_failed)
            .add_systems(
                Update,
                game_over_screen::handle_restart_button.run_if(in_state(AppState::GameOver)),
            );
    }
}
