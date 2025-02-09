use bevy::prelude::*;

#[derive(Resource)]
pub struct GameProgress {
    pub game_completed_counter: u32,
    pub death_counter: u32,
    pub total_career_level: u32,
    pub progress_points: u32,
}

impl Default for GameProgress {
    fn default() -> Self {
        GameProgress {
            game_completed_counter: 0,
            death_counter: 0,
            total_career_level: 0,
            progress_points: 5,
        }
    }
}
