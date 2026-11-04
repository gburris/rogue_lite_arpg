//! Publishes live projectile and status effect counts as `game/…` diagnostics.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;

use super::projectile::Projectile;
use super::status_effects::StatusOf;
use crate::prelude::*;

const PROJECTILES: DiagnosticPath = DiagnosticPath::const_new("game/projectiles");
/// Burn, freeze and slow effects applied to characters.
const STATUS_EFFECTS: DiagnosticPath = DiagnosticPath::const_new("game/status_effects");

pub(super) fn plugin(app: &mut App) {
    for path in [PROJECTILES, STATUS_EFFECTS] {
        app.register_diagnostic(Diagnostic::new(path).with_smoothing_factor(0.0));
    }
    app.add_systems(
        Update,
        (measure_projectiles, measure_status_effects).in_set(InGameSystems::HudOverlay),
    );
}

fn measure_projectiles(mut diagnostics: Diagnostics, projectiles: Query<(), With<Projectile>>) {
    diagnostics.add_measurement(&PROJECTILES, || projectiles.count() as f64);
}

fn measure_status_effects(mut diagnostics: Diagnostics, status_effects: Query<(), With<StatusOf>>) {
    diagnostics.add_measurement(&STATUS_EFFECTS, || status_effects.count() as f64);
}
