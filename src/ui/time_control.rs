use avian2d::prelude::*;
use bevy::prelude::*;

// Make pause menu visible when we enter the state
pub fn resume_game(mut time: ResMut<Time<Physics>>) {
    debug!("resume_game");
    time.unpause();
}

// Cleanup pause menu once we return to game, set it to hidden
pub fn pause_game(mut time: ResMut<Time<Physics>>) {
    debug!("pause_game");
    time.pause();
}
