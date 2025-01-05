use bevy::prelude::*;

use crate::{
    enemy::events::DespawnAllEnemies,
    labels::states::GameState,
    map::events::{DespawnAllPortals, DespawnAllTiles, StartRunEvent},
    projectile::events::DespawnAllProjectiles,
};

pub fn enter_start_portal(
    mut commands: Commands,
    mut events: EventReader<StartRunEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _event in events.read() {
        info!("Starting portal entered, beginning run!");
        commands.trigger(DespawnAllPortals);
        commands.trigger(DespawnAllTiles);
        commands.trigger(DespawnAllEnemies);
        commands.trigger(DespawnAllProjectiles);
        game_state.set(GameState::CreateZone);
    }
}
