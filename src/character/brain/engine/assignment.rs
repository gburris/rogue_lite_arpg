//! Fills the motion and action slots: skips what may not run, scores the rest, and
//! takes winners from the highest score down.

use super::behavior::{Behavior, BrainDefinition, MAX_BEHAVIORS};
use super::scoring::score_behavior;
use crate::character::brain::capability::AvailableMotionActionTable;
use crate::character::brain::perception::DecisionContext;

/// Score multiplier a behavior keeps over challengers while it holds a slot, so
/// a near-tie does not flip the character back and forth between ticks.
pub(crate) const MOMENTUM: f32 = 1.25;

/// Why a behavior sits out a decision unscored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipReason {
    /// A slot it claims is held by a committed behavior it does not outrank.
    SlotCommitted,
    /// This is the committed behavior holding its slots, so it is not re-scored.
    RunningCommitted,
    /// It may not start now: a missing capability, a cooldown, or the world.
    Unavailable,
}

impl SkipReason {
    /// Short text for the debug overlay.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SlotCommitted => "slot committed",
            Self::RunningCommitted => "running committed",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Scores every behavior not skipped (see [`SkipReason`]), then takes them
/// from the highest score down while their slots are free; ties go to the
/// earlier behavior. A `false` from `start` drops a winner for the next best.
/// Behavior ids, taken and returned, are indices into `definition.behaviors`.
pub fn assign_behaviors_to_slots(
    definition: &BrainDefinition,
    decision_context: &DecisionContext,
    available_motion_action_table: &AvailableMotionActionTable,
    running_behavior_ids: &[usize],
    mut start: impl FnMut(usize) -> bool,
) -> Vec<usize> {
    let mut scores = [0.0_f32; MAX_BEHAVIORS];
    for (index, behavior) in definition.behaviors.iter().enumerate() {
        if find_skip_reason(
            definition,
            behavior,
            available_motion_action_table,
            running_behavior_ids,
        )
        .is_some()
        {
            continue;
        }
        let is_current = running_behavior_ids.contains(&index);
        let Some(score) = score_behavior(behavior, decision_context, is_current) else {
            continue;
        };
        scores[index] = score * momentum_multiplier(is_current);
    }

    let mut winning_behavior_ids: Vec<usize> = Vec::with_capacity(2);
    loop {
        let mut best: Option<usize> = None;
        for (index, behavior) in definition.behaviors.iter().enumerate() {
            let taken = winning_behavior_ids.iter().any(|&winning_behavior_id| {
                definition.behaviors[winning_behavior_id].shares_a_slot_with(behavior)
            });
            if scores[index] <= 0.0 || taken {
                continue;
            }
            if best.is_none_or(|leader| scores[index] > scores[leader]) {
                best = Some(index);
            }
        }
        let Some(index) = best else {
            return winning_behavior_ids;
        };
        if !start(index) {
            scores[index] = 0.0;
            continue;
        }
        winning_behavior_ids.push(index);
    }
}

pub(super) fn find_skip_reason(
    definition: &BrainDefinition,
    behavior: &Behavior,
    available_motion_action_table: &AvailableMotionActionTable,
    running_behavior_ids: &[usize],
) -> Option<SkipReason> {
    let tier = behavior.commit.interrupt_tier();
    let slot_committed = running_behavior_ids.iter().any(|&running_behavior_id| {
        let running_behavior = &definition.behaviors[running_behavior_id];
        let running_tier = running_behavior.commit.interrupt_tier();
        running_tier > 0 && tier <= running_tier && running_behavior.shares_a_slot_with(behavior)
    });
    if slot_committed {
        return Some(SkipReason::SlotCommitted);
    }
    let unavailable = behavior
        .motion
        .is_some_and(|motion| !available_motion_action_table[motion])
        || behavior
            .action
            .is_some_and(|action| !available_motion_action_table[action]);
    unavailable.then_some(SkipReason::Unavailable)
}

pub(super) fn momentum_multiplier(is_current: bool) -> f32 {
    if is_current { MOMENTUM } else { 1.0 }
}
