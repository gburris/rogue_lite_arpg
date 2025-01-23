use bevy::prelude::*;

use crate::labels::states::AppState;

pub fn finish_create_instance(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::Playing);
}
