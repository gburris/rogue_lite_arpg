use bevy::prelude::*;

use crate::{
    enemy::events::DespawnAllEnemies,
    labels::states::{GameState, PlayingState},
    npc::events::DespawnAllNPCs,
    map::events::{DespawnAllPortals, DespawnAllTiles, StartRunEvent},
    projectile::events::DespawnAllProjectiles,
};

pub fn enter_start_portal(
    mut commands: Commands,
    mut events: EventReader<StartRunEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut playing_state: ResMut<NextState<PlayingState>>,
) {
    for _event in events.read() {
        commands.trigger(DespawnAllPortals);
        commands.trigger(DespawnAllTiles);
        commands.trigger(DespawnAllEnemies);
        commands.trigger(DespawnAllProjectiles);
        commands.trigger(DespawnAllNPCs);
        game_state.set(GameState::CreateZone);
        playing_state.set(PlayingState::Run);
    }
}
