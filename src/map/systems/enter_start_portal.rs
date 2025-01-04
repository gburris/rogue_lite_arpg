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
        warn!("Collision with run portal event processing!");

        commands.trigger(DespawnAllPortals);
        commands.trigger(DespawnAllTiles);
        commands.trigger(DespawnAllEnemies);
        commands.trigger(DespawnAllProjectiles);
        game_state.set(GameState::CreateZone);
        debug!("Starting the rouge-like run!");
    }
}
