use bevy::prelude::*;

use crate::{labels::states::AppState, player::systems::PauseInputEvent};

pub fn handle_ui_inputs(mut commands: Commands, mut keyboard_input: ResMut<ButtonInput<KeyCode>>) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        warn!("ui_inputs, enter");
        commands.trigger(PauseInputEvent);
    }
}

pub fn on_pause_input(
    _: Trigger<PauseInputEvent>, // Access keyboard input
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match state.get() {
        AppState::Paused => {
            warn!("Currently paused, transitioning to playing");
            next_state.set(AppState::Playing)
        }
        _ => {
            warn!("Not currently paused, transitioning to paused");
            next_state.set(AppState::Paused);
        }
    }
}
