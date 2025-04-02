use bevy::prelude::*;

use crate::{
    controller::plugin::PauseInputEvent,
    labels::states::{AppState, PausedState},
};

pub fn on_pause_input_event(
    pause_event: Trigger<PauseInputEvent>,
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
            if let PauseInputEvent(Some(state)) = *pause_event {
                next_pause_state.set(state);
            }
        }
    }
}
