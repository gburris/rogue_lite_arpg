//! Publishes brain and decision counts as `game/ai/…` diagnostics.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;

use super::{Brain, Targeting};
use crate::prelude::*;

const BRAINS: DiagnosticPath = DiagnosticPath::const_new("game/ai/brains");
/// Brains with a target. More of them means a heavier fight.
const ENGAGED: DiagnosticPath = DiagnosticPath::const_new("game/ai/engaged");
const DECISIONS: DiagnosticPath = DiagnosticPath::const_new("game/ai/decisions");
const REDECISIONS: DiagnosticPath = DiagnosticPath::const_new("game/ai/redecisions");

/// Decisions made since the last frame was published.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub(super) struct DecisionStats {
    paced_decisions: u32,
    redecisions: u32,
}

impl DecisionStats {
    /// Counts a decision made on a brain's 10 Hz schedule.
    pub(super) fn record_paced_decision(&mut self) {
        self.paced_decisions += 1;
    }

    /// Counts a decision an observer made immediately, outside the 10 Hz schedule.
    pub(super) fn record_redecision(&mut self) {
        self.redecisions += 1;
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DecisionStats>();
    for path in [BRAINS, ENGAGED, DECISIONS, REDECISIONS] {
        app.register_diagnostic(Diagnostic::new(path).with_smoothing_factor(0.0));
    }
    app.add_systems(
        Update,
        (measure_brains, publish_decision_stats).in_set(InGameSystems::Measure),
    );
}

fn measure_brains(
    mut diagnostics: Diagnostics,
    brains: Query<(), With<Brain>>,
    engaged_brains: Query<(), (With<Brain>, With<Targeting>)>,
) {
    diagnostics.add_measurement(&BRAINS, || brains.count() as f64);
    diagnostics.add_measurement(&ENGAGED, || engaged_brains.count() as f64);
}

/// Publishes the decisions made since the previous frame, then resets the counts.
fn publish_decision_stats(mut diagnostics: Diagnostics, mut decision_stats: ResMut<DecisionStats>) {
    let decisions = f64::from(decision_stats.paced_decisions);
    let redecisions = f64::from(decision_stats.redecisions);
    diagnostics.add_measurement(&DECISIONS, || decisions);
    diagnostics.add_measurement(&REDECISIONS, || redecisions);

    *decision_stats = DecisionStats::default();
}
