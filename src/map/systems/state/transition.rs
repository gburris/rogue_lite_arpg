use bevy::prelude::*;

use crate::labels::states::AppState;

//TODO:
//This is a poor system bounder, and this function is only called in the
//Player crate.
//I think we need a "Transitions" domain or something for all these calls.
pub fn transition_to_create_hub(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::CreateHub);
}
