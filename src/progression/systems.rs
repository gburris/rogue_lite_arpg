use bevy::prelude::*;

use crate::configuration::time::RestartEvent;

use super::GameProgress;

/// Triggers when restart is clicked after death in a run
pub fn handle_restart_trigger(
    restart_event_trigger: Trigger<RestartEvent>,
    mut game_progress: ResMut<GameProgress>,
) {
    game_progress.death_counter += 1;
    game_progress.total_career_level += restart_event_trigger.player_level;
    game_progress.progress_points += restart_event_trigger.player_level;
}
