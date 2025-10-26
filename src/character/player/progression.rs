use bevy::prelude::*;

use crate::prelude::{AppState, CleanupZone, PlayerStats, RestartEvent};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(GameProgress::default());

    app.add_observer(handle_restart_trigger);
}

#[derive(Resource)]
pub struct GameProgress {
    pub game_completed_counter: u32,
    pub death_counter: u32,
    pub total_career_level: u32,
    pub progress_points: u32,
    pub base_stats: PlayerStats, //Base stats are upgraded at the NPC each run
}

impl Default for GameProgress {
    fn default() -> Self {
        GameProgress {
            game_completed_counter: 0,
            death_counter: 0,
            total_career_level: 0,
            progress_points: 5,
            base_stats: PlayerStats::default(),
        }
    }
}

/// Triggers when restart is clicked after death in a run
fn handle_restart_trigger(
    restart_event_trigger: On<RestartEvent>,
    mut commands: Commands,
    mut game_progress: ResMut<GameProgress>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    game_progress.death_counter += 1;
    game_progress.total_career_level += restart_event_trigger.player_level;
    game_progress.progress_points += restart_event_trigger.player_level;

    commands.trigger(CleanupZone);
    next_app_state.set(AppState::SpawnPlayer);
}
