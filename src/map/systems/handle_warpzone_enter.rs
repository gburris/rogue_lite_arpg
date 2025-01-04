use bevy::prelude::*;

use crate::{
    enemy::events::DespawnAllEnemies,
    labels::states::GameState,
    map::events::{DespawnAllPortals, DespawnAllTiles, WarpZoneEnterEvent},
    player::ResetPlayerPosition,
    projectile::events::DespawnAllProjectiles,
};

pub fn handle_warpzone_enter(
    mut commands: Commands,
    mut events: EventReader<WarpZoneEnterEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _event in events.read() {
        commands.trigger(DespawnAllPortals);
        commands.trigger(DespawnAllTiles);
        commands.trigger(DespawnAllEnemies);
        commands.trigger(DespawnAllProjectiles);
        commands.trigger(ResetPlayerPosition);
        game_state.set(GameState::CreateZone);
    }
}
