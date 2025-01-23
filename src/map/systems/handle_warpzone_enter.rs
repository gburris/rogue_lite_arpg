use bevy::prelude::*;

use crate::{
    despawn::events::CleanupZone, labels::states::AppState, map::events::WarpZoneEnterEvent,
};

pub fn handle_warpzone_enter(
    mut commands: Commands,
    mut events: EventReader<WarpZoneEnterEvent>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for _event in events.read() {
        commands.trigger(CleanupZone);
        game_state.set(AppState::CreateInstance);
    }
}
