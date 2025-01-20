//Update the game progress resources based on the player level
//Every player level = One Progress Point
//Death Counter++;
//Triggers when restart is clicked using an observer, that trigger will have the current level value in it
use bevy::prelude::*;

use crate::ui::game_over_screen::RestartEvent;

use super::GameProgress;

pub fn handle_restart_trigger(
    restart_event_trigger: Trigger<RestartEvent>,
    mut game_progress: ResMut<GameProgress>,
) {
    game_progress.death_counter += 1;
    game_progress.total_career_level += restart_event_trigger.player_level;
    game_progress.progress_points += restart_event_trigger.player_level;
}
