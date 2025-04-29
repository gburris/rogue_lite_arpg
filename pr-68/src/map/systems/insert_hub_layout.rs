use bevy::prelude::*;

use crate::{labels::states::AppState, map::helpers::generator::generate_hub_layout};

pub fn insert_hub_layout(mut commands: Commands, mut game_state: ResMut<NextState<AppState>>) {
    let map_layout = generate_hub_layout();
    commands.insert_resource(map_layout);
    game_state.set(AppState::SpawnZone);
}
