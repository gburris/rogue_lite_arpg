use bevy::prelude::*;

use crate::{map::helpers::generator::generate_hub_layout, prelude::AppState};

pub fn insert_hub_layout(mut commands: Commands, mut game_state: ResMut<NextState<AppState>>) {
    let map_layout = generate_hub_layout();
    commands.insert_resource(map_layout);
    game_state.set(AppState::SpawnZone);
}
