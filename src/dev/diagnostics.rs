//! Publishes physics, lighting and asset counts as `game/…` diagnostics.

use avian2d::prelude::{ContactGraph, RayCaster, RigidBody};
use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;
use bevy_lit::prelude::PointLight2d;

use crate::prelude::*;

const PHYSICS_BODIES: DiagnosticPath = DiagnosticPath::const_new("game/physics/bodies");
const CONTACT_PAIRS: DiagnosticPath = DiagnosticPath::const_new("game/physics/contact_pairs");
const ENABLED_RAYS: DiagnosticPath = DiagnosticPath::const_new("game/physics/enabled_rays");
/// Fixed steps run since the last frame.
pub(super) const FIXED_STEPS: DiagnosticPath =
    DiagnosticPath::const_new("game/physics/fixed_steps");
const LIGHTS: DiagnosticPath = DiagnosticPath::const_new("game/lights");
const LOADED_IMAGES: DiagnosticPath = DiagnosticPath::const_new("game/images");

/// Fixed steps run since the last frame was published.
#[derive(Resource, Default)]
struct FixedStepCount {
    steps: u32,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FixedStepCount>();
    for path in [
        PHYSICS_BODIES,
        CONTACT_PAIRS,
        ENABLED_RAYS,
        FIXED_STEPS,
        LIGHTS,
        LOADED_IMAGES,
    ] {
        app.register_diagnostic(Diagnostic::new(path).with_smoothing_factor(0.0));
    }
    app.add_systems(FixedPostUpdate, count_fixed_step);
    app.add_systems(
        Update,
        (
            measure_physics_bodies,
            measure_contact_pairs,
            measure_enabled_rays,
            publish_fixed_steps,
            measure_lights,
            measure_loaded_images,
        )
            .in_set(InGameSystems::HudOverlay),
    );
}

fn count_fixed_step(mut fixed_step_count: ResMut<FixedStepCount>) {
    fixed_step_count.steps += 1;
}

fn measure_physics_bodies(mut diagnostics: Diagnostics, bodies: Query<(), With<RigidBody>>) {
    diagnostics.add_measurement(&PHYSICS_BODIES, || bodies.count() as f64);
}

fn measure_contact_pairs(mut diagnostics: Diagnostics, contact_graph: Res<ContactGraph>) {
    diagnostics.add_measurement(&CONTACT_PAIRS, || contact_graph.active_pairs().len() as f64);
}

fn measure_enabled_rays(mut diagnostics: Diagnostics, ray_casters: Query<&RayCaster>) {
    diagnostics.add_measurement(&ENABLED_RAYS, || {
        ray_casters
            .iter()
            .filter(|ray_caster| ray_caster.enabled)
            .count() as f64
    });
}

/// Publishes the fixed steps run since the previous frame, then resets the count.
fn publish_fixed_steps(mut diagnostics: Diagnostics, mut fixed_step_count: ResMut<FixedStepCount>) {
    let steps = f64::from(fixed_step_count.steps);
    diagnostics.add_measurement(&FIXED_STEPS, || steps);
    fixed_step_count.steps = 0;
}

fn measure_lights(mut diagnostics: Diagnostics, lights: Query<(), With<PointLight2d>>) {
    diagnostics.add_measurement(&LIGHTS, || lights.count() as f64);
}

fn measure_loaded_images(mut diagnostics: Diagnostics, images: Res<Assets<Image>>) {
    diagnostics.add_measurement(&LOADED_IMAGES, || images.len() as f64);
}
