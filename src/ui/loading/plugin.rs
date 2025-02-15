use bevy::prelude::*;

use crate::labels::states::AppState;

use super::load_screen::{animate_text, spawn_load_screen, LoadScreen};
use crate::despawn::systems::*;

pub struct LoadingUIPlugin;

impl Plugin for LoadingUIPlugin {
    fn build(&self, app: &mut App) {
        // Game UI systems
        app.add_systems(OnEnter(AppState::SpawnZone), spawn_load_screen)
            .add_systems(
                OnExit(AppState::SpawnZone),
                despawn_single::<LoadScreen>,
            )
            .add_systems(
                Update,
                (animate_text).run_if(in_state(AppState::SpawnZone)),
            );
    }
}
