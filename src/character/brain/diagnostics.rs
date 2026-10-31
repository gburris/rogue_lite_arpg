//! Publishes the number of live brains as `game/ai/brains`.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;

use super::Brain;
use crate::prelude::*;

const BRAINS: DiagnosticPath = DiagnosticPath::const_new("game/ai/brains");

pub(super) fn plugin(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(BRAINS).with_smoothing_factor(0.0));
    app.add_systems(Update, measure_brains.in_set(InGameSystems::HudOverlay));
}

fn measure_brains(mut diagnostics: Diagnostics, brains: Query<(), With<Brain>>) {
    diagnostics.add_measurement(&BRAINS, || brains.count() as f64);
}
