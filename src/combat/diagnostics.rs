//! Publishes the number of live projectiles as `game/projectiles`.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;

use super::projectile::Projectile;
use crate::prelude::*;

const PROJECTILES: DiagnosticPath = DiagnosticPath::const_new("game/projectiles");

pub(super) fn plugin(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(PROJECTILES).with_smoothing_factor(0.0));
    app.add_systems(
        Update,
        measure_projectiles.in_set(InGameSystems::HudOverlay),
    );
}

fn measure_projectiles(mut diagnostics: Diagnostics, projectiles: Query<(), With<Projectile>>) {
    diagnostics.add_measurement(&PROJECTILES, || projectiles.count() as f64);
}
