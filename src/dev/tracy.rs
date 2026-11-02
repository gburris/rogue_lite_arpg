//! Sends `game/…` diagnostics and state changes to Tracy as plots and messages.

use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::state::{StateTransitionEvent, States};
use tracy_client::PlotName;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            send_state_message::<AppState>,
            send_state_message::<PlayingState>,
            send_state_message::<PauseState>,
            send_state_message::<Menu>,
        ),
    );
    app.add_systems(
        Update,
        plot_game_diagnostics.after(InGameSystems::HudOverlay),
    );
}

/// Plots the latest value of each `game/…` diagnostic.
fn plot_game_diagnostics(
    diagnostics_store: Res<DiagnosticsStore>,
    mut plot_names: Local<HashMap<DiagnosticPath, PlotName>>,
) {
    let Some(client) = tracy_client::Client::running() else {
        return;
    };
    for diagnostic in diagnostics_store.iter() {
        let path = diagnostic.path();
        if !path.as_str().starts_with("game/") {
            continue;
        }
        let Some(value) = diagnostic.value() else {
            continue;
        };
        let plot_name = *plot_names
            .entry(path.clone())
            .or_insert_with(|| PlotName::new_leak(path.to_string()));
        client.plot(plot_name, value);
    }
}

/// Pins each state change to the capture's timeline, such as the Start click moving
/// `AppState` out of `StartScreen`.
fn send_state_message<S: States>(mut transitions: MessageReader<StateTransitionEvent<S>>) {
    let Some(client) = tracy_client::Client::running() else {
        return;
    };
    for transition in transitions.read() {
        let state_name = core::any::type_name::<S>()
            .rsplit("::")
            .next()
            .expect("type names are never empty");
        client.message(
            &format!(
                "{state_name}: {:?} => {:?}",
                transition.exited, transition.entered
            ),
            0,
        );
    }
}
