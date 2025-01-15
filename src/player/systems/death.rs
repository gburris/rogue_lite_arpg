use bevy::prelude::*;

use crate::{
    combat::damage::events::DefeatedEvent, despawn::events::CleanupCurrentWorldSpace,
    labels::states::AppState,
};

pub fn on_player_defeated(
    _: Trigger<DefeatedEvent>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
) {
    // TODO: Add death animation
    commands.trigger(CleanupCurrentWorldSpace);
    game_state.set(AppState::GameOver);
}
