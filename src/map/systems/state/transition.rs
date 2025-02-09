use bevy::prelude::*;

use crate::labels::states::AppState;

//TODO: Move all of the state transition in map/ to this file
pub fn transition_to_create_hub(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::CreateHub);
}
