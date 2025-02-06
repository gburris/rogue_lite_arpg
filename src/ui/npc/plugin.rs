use bevy::prelude::*;

use crate::labels::{sets::MainSet, states::PausedState};

use super::{
    handle_stats_shop_interaction::{
        handle_player_stat_change, handle_stat_button_interaction, handle_stats_shop_ui_update,
    },
    stats_shop::{despawn_stats_shop_menu, spawn_stats_shop_menu},
};

pub struct NPCPauseScreensPlugin;

impl Plugin for NPCPauseScreensPlugin {
    fn build(&self, app: &mut App) {
        app
            // Pause Related Systems
            .add_observer(handle_player_stat_change)
            .add_observer(handle_stats_shop_ui_update)
            .add_systems(OnEnter(PausedState::StatsShop), spawn_stats_shop_menu)
            .add_systems(OnExit(PausedState::StatsShop), despawn_stats_shop_menu)
            .add_systems(
                Update,
                handle_stat_button_interaction
                    .run_if(in_state(PausedState::StatsShop))
                    .in_set(MainSet::Menu),
            );
    }
}
