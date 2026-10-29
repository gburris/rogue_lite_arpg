//! The pure utility scoring engine: behaviors, static brain definitions, and
//! the math that fills the motion and action slots. No ECS and no content.

mod scoring;

pub use scoring::{Consideration, Curve, Interval, ScoreMapping};

use super::action::Action;
use super::capability::AvailableMotionActionTable;
use super::motion::Motion;
use super::perception::{ConsiderationInput, DecisionContext, read_consideration_input};

/// Score multiplier a behavior keeps over challengers while it holds a slot, so
/// a near-tie does not flip the character back and forth between ticks.
pub(crate) const MOMENTUM: f32 = 1.25;

/// Most behaviors one brain definition may list, so scoring needs no allocation.
pub(crate) const MAX_BEHAVIORS: usize = 16;

/// One candidate a brain can choose: a motion, an action, or both, and how
/// firmly it holds them.
#[derive(Clone, Copy, Debug)]
pub struct Behavior {
    pub motion: Option<Motion>,
    pub action: Option<Action>,
    pub commit: CommitPolicy,
    pub weight: f32,
    pub considerations: &'static [Consideration],
}

impl Behavior {
    /// Builds a behavior; [`validate_brain`] checks its shape and weight.
    pub const fn new(
        motion: Option<Motion>,
        action: Option<Action>,
        commit: CommitPolicy,
        weight: f32,
        considerations: &'static [Consideration],
    ) -> Self {
        Self {
            motion,
            action,
            commit,
            weight,
            considerations,
        }
    }

    /// Whether both behaviors claim the motion slot or both claim the action slot.
    pub fn shares_a_slot_with(&self, other: &Behavior) -> bool {
        (self.motion.is_some() && other.motion.is_some())
            || (self.action.is_some() && other.action.is_some())
    }
}

/// How firmly a behavior holds its slots once it wins them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommitPolicy {
    /// Re-scored every tick with momentum and exit-side thresholds. Tier 0.
    ContinuouslyScored,
    /// Not re-scored until its finite part finishes or the brain drops it
    /// (target lost, stunned, or timed out), so its start conditions are read
    /// once. Only a committed behavior with a higher `interrupt_tier` may take
    /// one of its slots, which ends the whole behavior.
    UntilFinished { interrupt_tier: u8 },
}

impl CommitPolicy {
    pub const fn is_committed(self) -> bool {
        matches!(self, Self::UntilFinished { .. })
    }

    /// The tier a holder must be outranked by to lose its slots; 0 for a continuously scored behavior.
    pub const fn interrupt_tier(self) -> u8 {
        match self {
            Self::UntilFinished { interrupt_tier } => interrupt_tier,
            Self::ContinuouslyScored => 0,
        }
    }
}

/// A complete static brain: the list of behaviors it scores.
#[derive(Debug)]
pub struct BrainDefinition {
    pub behaviors: &'static [Behavior],
}

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

/// Final bounded score for one behavior. `is_current` switches threshold
/// considerations to their exit side.
pub fn score_behavior(
    behavior: &Behavior,
    decision_context: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // Weight times the geometric mean: a zero anywhere vetoes, but each
    // passing flag raises graded inputs to a higher root, so adding a flag
    // shifts a behavior's crossovers. Brains are tuned against that root.
    let mut product = 1.0;
    let mut count = 0;
    for consideration in behavior.considerations {
        product *= score_consideration(consideration, decision_context, is_current)?;
        count += 1;
    }
    let mut score = behavior.weight;
    if count > 0 {
        score *= product.powf(1.0 / (count as f32));
    }

    (score.is_finite() && score >= 0.0).then_some(score)
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

/// Rejects brains that compile but cannot behave. Intervals and thresholds
/// are checked by their constructors at compile time.
pub fn validate_brain(brain: &BrainDefinition) -> Result<(), String> {
    if brain.behaviors.is_empty() {
        return Err("behavior list is empty".to_string());
    }
    if brain.behaviors.len() > MAX_BEHAVIORS {
        return Err(format!("more than {MAX_BEHAVIORS} behaviors"));
    }
    for (index, behavior) in brain.behaviors.iter().enumerate() {
        if behavior.motion.is_none() && behavior.action.is_none() {
            return Err(format!("behavior {index} claims no slot"));
        }
        if !(behavior.weight.is_finite() && behavior.weight > 0.0) {
            return Err(format!(
                "behavior {index} weight must be finite and positive"
            ));
        }
        let committed = behavior.commit.is_committed();
        if committed
            && behavior
                .action
                .is_some_and(|action| !action.reports_finish())
        {
            return Err(format!(
                "behavior {index} commits to an action that never reports a finish"
            ));
        }
        if committed && behavior.commit.interrupt_tier() == 0 {
            return Err(format!(
                "behavior {index} is committed in tier 0, which is for continuously scored behaviors"
            ));
        }
        // Every other action is finite, so any action is a finite part.
        let finite_motion = behavior.motion.is_some_and(Motion::is_finite);
        let finite_parts = usize::from(finite_motion) + usize::from(behavior.action.is_some());
        if committed && finite_parts != 1 {
            return Err(format!(
                "behavior {index} is committed with {finite_parts} finite parts; one finish must end it"
            ));
        }
        if !committed && finite_motion {
            return Err(format!(
                "behavior {index} has a finite motion but is not committed"
            ));
        }
    }
    Ok(())
}

/// Explains behavior `index` exactly as [`assign_behaviors_to_slots`] scores
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

fn find_skip_reason(
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

fn momentum_multiplier(is_current: bool) -> f32 {
    if is_current { MOMENTUM } else { 1.0 }
}

fn score_consideration(
    consideration: &Consideration,
    decision_context: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // A missing input (no target, no home area) vetoes, inverted or not.
    let Some(raw) = read_consideration_input(consideration.input, decision_context) else {
        return Some(0.0);
    };
    let score = consideration
        .score_mapping
        .score_raw_input(raw, is_current)?;
    Some(if consideration.inverted {
        1.0 - score
    } else {
        score
    })
}
