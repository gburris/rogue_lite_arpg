//! The behaviors a brain is running and how each one finishes.

use bevy::prelude::*;

use super::perception::DecisionContext;
use super::{Action, ActionFinished, ActionOutcome, Motion};

/// Longest a committed behavior may run, a safety net against a missing finish.
const COMMIT_TIMEOUT: f32 = 3.0;

/// A behavior that won its slots, with its motion and action copied from the
/// definition and stamped with a run id so a finite motion's
/// `CurrentBehaviorRunId` can tell a fresh
/// start from the run it just finished. Appliers report through it, so the
/// committed/continuously scored finish policy lives here.
#[derive(Clone, Copy, Debug)]
pub(super) struct RunningBehavior {
    pub(super) behavior: usize,
    pub(super) motion: Option<Motion>,
    pub(super) action: Option<Action>,
    pub(super) behavior_run_id: u32,
    pub(super) committed: bool,
    pub(super) started_at: f32,
}

impl RunningBehavior {
    pub(super) fn report_completed(self, commands: &mut Commands, entity: Entity) {
        if let Some(finished) = self.build_action_finished(entity, ActionOutcome::Completed) {
            commands.trigger(finished);
        }
    }

    /// Ends a committed use the live world refused; a continuously scored use waits.
    pub(super) fn report_not_started(self, commands: &mut Commands, entity: Entity) {
        if self.committed
            && let Some(finished) = self.build_action_finished(entity, ActionOutcome::NotStarted)
        {
            commands.trigger(finished);
        }
    }

    /// `None` for a record with no action to report on.
    fn build_action_finished(
        self,
        entity: Entity,
        outcome: ActionOutcome,
    ) -> Option<ActionFinished> {
        Some(ActionFinished {
            entity,
            action: self.action?,
            behavior_run_id: self.behavior_run_id,
            outcome,
        })
    }

    pub(super) fn should_drop_committed_behavior(
        &self,
        context: &DecisionContext,
        now: f32,
    ) -> bool {
        self.committed
            && (context.target_view.is_none()
                || context.self_view.stunned
                || now - self.started_at > COMMIT_TIMEOUT)
    }
}
