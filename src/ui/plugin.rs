use bevy::prelude::*;

use crate::{prelude::*, ui::*};

use super::{npc::plugin::NPCPauseScreensPlugin, pause_menu::plugin::PauseMenuPlugin};

/// Plugin responsible for managing all UI-related systems and state transitions
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // sub UI plugins
        app.add_plugins(PauseMenuPlugin)
            .add_plugins(NPCPauseScreensPlugin);

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

        // Core game overlay (HUD)
        app.add_systems(OnEnter(AppState::SpawnPlayer), player_overlay::spawn)
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
                    .in_set(InGameSystems::HudOverlay),
            )
            .add_observer(damage_overlay::on_damage_overlay_amount)
            .add_observer(damage_overlay::on_healing_overlay_amount)
            .add_observer(player_overlay::on_equipment_used)
            .add_observer(player_overlay::on_equipment_use_failed)
            .add_observer(despawn_all::<RestartEvent, PlayerOverlay>);

        // Game over systems
        app.add_systems(OnEnter(AppState::GameOver), game_over_screen::spawn);
    }
}
