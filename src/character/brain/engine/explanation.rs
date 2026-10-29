//! Debug explanations of how a behavior scored, computed from the same inputs as
//! slot assignment.

use super::assignment::{SkipReason, find_skip_reason, momentum_multiplier};
use super::behavior::BrainDefinition;
use super::scoring::{score_behavior, score_consideration};
use crate::character::brain::capability::AvailableMotionActionTable;
use crate::character::brain::perception::{
    ConsiderationInput, DecisionContext, read_consideration_input,
};

/// One consideration's part in a score. `raw` is `None` when the input is
/// absent; `score` is `None` when the score mapping rejects the raw value.
#[derive(Clone, Copy, Debug)]
pub struct ConsiderationScoreExplanation {
    pub name: &'static str,
    pub input: ConsiderationInput,
    pub raw: Option<f32>,
    pub score: Option<f32>,
}

/// How one behavior scored, for the debug trace: why it sat out, or its final
/// score with `momentum` applied (`None` when invalid).
#[derive(Clone, Debug)]
pub struct BehaviorScoreExplanation {
    pub skipped: Option<SkipReason>,
    pub score: Option<f32>,
    pub momentum: f32,
    pub considerations: Vec<ConsiderationScoreExplanation>,
}

/// Explains behavior `index` exactly as [`assign_behaviors_to_slots`](super::assignment::assign_behaviors_to_slots) scores
/// it from the same inputs.
pub fn explain_behavior(
    definition: &BrainDefinition,
    index: usize,
    decision_context: &DecisionContext,
    available_motion_action_table: &AvailableMotionActionTable,
    running_behavior_ids: &[usize],
) -> BehaviorScoreExplanation {
    let behavior = &definition.behaviors[index];
    let is_current = running_behavior_ids.contains(&index);
    let momentum = momentum_multiplier(is_current);
    let skipped = find_skip_reason(
        definition,
        behavior,
        available_motion_action_table,
        running_behavior_ids,
    )
    .map(|reason| {
        // The committed holder sits out too, but it is the one running.
        match reason {
            SkipReason::SlotCommitted if is_current => SkipReason::RunningCommitted,
            other => other,
        }
    });
    if skipped.is_some() {
        return BehaviorScoreExplanation {
            skipped,
            score: None,
            momentum,
            considerations: Vec::new(),
        };
    }
    let considerations = behavior
        .considerations
        .iter()
        .map(|consideration| ConsiderationScoreExplanation {
            name: consideration.name,
            input: consideration.input,
            raw: read_consideration_input(consideration.input, decision_context),
            score: score_consideration(consideration, decision_context, is_current),
        })
        .collect();
    BehaviorScoreExplanation {
        skipped,
        score: score_behavior(behavior, decision_context, is_current).map(|score| score * momentum),
        momentum,
        considerations,
    }
}
