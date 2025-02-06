use bevy::prelude::*;

use crate::{
    labels::states::{AppState, PausedState},
    player::systems::PauseInputEvent,
};

//UN-Pause logic, runs when App State is Paused
pub fn handle_ui_inputs(mut commands: Commands, mut keyboard_input: ResMut<ButtonInput<KeyCode>>) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        debug!("ui_inputs, enter");
        commands.trigger(PauseInputEvent { paused_state: None });
    }
}

pub fn on_pause_input(
    pause_input_trigger: Trigger<PauseInputEvent>,
    mut next_pause_state: ResMut<NextState<PausedState>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match state.get() {
        AppState::Paused => {
            debug!("Currently paused, transitioning to playing");
            next_state.set(AppState::Playing)
        }
        _ => {
            debug!("Not currently paused, transitioning to paused");
            next_state.set(AppState::Paused);
            if let Some(paused_state) = pause_input_trigger.paused_state {
                next_pause_state.set(paused_state);
            }
        }
    }
}
