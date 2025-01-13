use bevy::prelude::*;

use crate::{
    combat::damage::events::DefeatedEvent, despawn::events::CleanupCurrentWorldSpace,
    labels::states::GameState,
};

pub fn on_player_defeated(
    _: Trigger<DefeatedEvent>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
) {
    // TODO: Add death animation
    commands.trigger(CleanupCurrentWorldSpace);
    game_state.set(GameState::GameOver);
}
