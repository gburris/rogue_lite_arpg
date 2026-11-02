//! Mirrors every `game/…` diagnostic into a Tracy plot of the same name.

use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use tracy_client::PlotName;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
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
