//! Publishes physics and asset counts as `game/physics/bodies` and `game/images`.

use avian2d::prelude::RigidBody;
use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;

use crate::prelude::*;

const PHYSICS_BODIES: DiagnosticPath = DiagnosticPath::const_new("game/physics/bodies");
const LOADED_IMAGES: DiagnosticPath = DiagnosticPath::const_new("game/images");

pub(super) fn plugin(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(PHYSICS_BODIES).with_smoothing_factor(0.0));
    app.register_diagnostic(Diagnostic::new(LOADED_IMAGES).with_smoothing_factor(0.0));
    app.add_systems(
        Update,
        (measure_physics_bodies, measure_loaded_images).in_set(InGameSystems::HudOverlay),
    );
}

fn measure_physics_bodies(mut diagnostics: Diagnostics, bodies: Query<(), With<RigidBody>>) {
    diagnostics.add_measurement(&PHYSICS_BODIES, || bodies.count() as f64);
}

fn measure_loaded_images(mut diagnostics: Diagnostics, images: Res<Assets<Image>>) {
    diagnostics.add_measurement(&LOADED_IMAGES, || images.len() as f64);
}
