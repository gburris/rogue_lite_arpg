use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(GameProgress::default());

    app.add_observer(on_restart);
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
        Self {
            game_completed_counter: 0,
            death_counter: 0,
            total_career_level: 0,
            progress_points: 5,
            base_stats: PlayerStats::default(),
        }
    }
}

/// Triggers when restart is clicked after death in a run.
fn on_restart(
    restart: On<Restart>,
    mut game_progress: ResMut<GameProgress>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    game_progress.death_counter += 1;
    game_progress.total_career_level += restart.player_level;
    game_progress.progress_points += restart.player_level;

    next_app_state.set(AppState::SpawnPlayer);
}
