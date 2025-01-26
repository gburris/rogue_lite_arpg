use bevy::prelude::*;

use crate::{
    despawn::events::CleanupZone, labels::states::AppState, map::events::CreateInstanceEvent,
};

pub fn on_portal_entered(
    _: Trigger<CreateInstanceEvent>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
) {
    commands.trigger(CleanupZone);
    game_state.set(AppState::CreateInstance);
}
