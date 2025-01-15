use bevy::prelude::*;

use crate::{
    despawn::events::CleanupCurrentWorldSpace,
    labels::states::{AppState, InGameState},
    map::events::StartRunEvent,
};

pub fn enter_start_portal(
    mut commands: Commands,
    mut events: EventReader<StartRunEvent>,
    mut game_state: ResMut<NextState<AppState>>,
    mut playing_state: ResMut<NextState<InGameState>>,
) {
    for _event in events.read() {
        commands.trigger(CleanupCurrentWorldSpace);
        game_state.set(AppState::CreateZone);
        playing_state.set(InGameState::Run);
    }
}
