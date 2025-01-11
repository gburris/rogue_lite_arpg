use bevy::prelude::*;

use crate::{
    despawn::events::CleanupCurrentWorldSpace,
    labels::states::{GameState, PlayingState},
    map::events::StartRunEvent,
};

pub fn enter_start_portal(
    mut commands: Commands,
    mut events: EventReader<StartRunEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut playing_state: ResMut<NextState<PlayingState>>,
) {
    for _event in events.read() {
        commands.trigger(CleanupCurrentWorldSpace);
        game_state.set(GameState::CreateZone);
        playing_state.set(PlayingState::Run);
    }
}
