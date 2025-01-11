use bevy::prelude::*;

use crate::{
    despawn::events::CleanupCurrentWorldSpace, labels::states::GameState,
    map::events::WarpZoneEnterEvent,
};

pub fn handle_warpzone_enter(
    mut commands: Commands,
    mut events: EventReader<WarpZoneEnterEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _event in events.read() {
        commands.trigger(CleanupCurrentWorldSpace);
        game_state.set(GameState::CreateZone);
    }
}
