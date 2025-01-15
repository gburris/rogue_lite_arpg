use bevy::prelude::*;

use crate::{
    despawn::events::CleanupCurrentWorldSpace, labels::states::AppState,
    map::events::WarpZoneEnterEvent,
};

pub fn handle_warpzone_enter(
    mut commands: Commands,
    mut events: EventReader<WarpZoneEnterEvent>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for _event in events.read() {
        commands.trigger(CleanupCurrentWorldSpace);
        game_state.set(AppState::CreateZone);
    }
}
