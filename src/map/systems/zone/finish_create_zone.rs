use bevy::prelude::*;

use crate::{map::CleanupZone, prelude::AppState};

pub fn despawn_previous_zone(mut commands: Commands) {
    commands.trigger(CleanupZone);
}

pub fn finish_create_zone(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::Playing);
}
