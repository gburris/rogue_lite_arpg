use avian2d::prelude::*;
use bevy::prelude::*;

use crate::labels::states::AppState;

// Make pause menu visible when we enter the state
pub fn resume_game(mut time: ResMut<Time<Physics>>, mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Playing);
    warn!("resume_game");
    time.unpause();
}

// Cleanup pause menu once we return to game, set it to hidden
pub fn pause_game(mut time: ResMut<Time<Physics>>) {
    warn!("pause_game");
    time.pause();
}
