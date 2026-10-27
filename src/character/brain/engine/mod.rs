//! The utility scoring engine: what a brain can choose ([`Behavior`]), the
//! static definitions it scores against ([`BrainDefinition`]), and the math
//! that fills its slots. Every behavior scores its weight times the geometric
//! mean of its considerations. Behaviors then take the motion and action slots
//! greedily from the highest score down; a behavior that already holds a slot
//! scores with [`MOMENTUM`]. A slot held by a committed behavior is open only
//! to a committed behavior with a higher interrupt tier. A slot nothing scores
//! for stays empty.
//!
//! Pure data and math, no ECS and no content: it knows no particular brain or
//! item. It does name the brain's shared vocabulary, the [`Motion`] and
//! [`Action`] a behavior claims and the inputs its considerations read through
//! [`read_input`], because behaviors are static data built from those types.

mod response;

pub use response::{Consideration, Curve, Interval, Response};

use super::action::Action;
use super::capability::Availability;
use super::motion::Motion;
use super::perception::{ConsiderationInput, DecisionContext, read_input};

/// Score multiplier a behavior keeps over challengers while it holds a slot, so
/// a near-tie does not flip the character back and forth between ticks.
pub(crate) const MOMENTUM: f32 = 1.25;

/// Most behaviors one brain definition may list, so scoring needs no allocation.
pub(crate) const MAX_BEHAVIORS: usize = 16;

/// One candidate a brain can choose: how the character moves, what it does,
/// and how firmly it holds on, scored as its weight times the geometric mean
/// of its considerations. A behavior claims the motion slot, the action slot,
/// or both.
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
}

/// How firmly a behavior holds its slots once it wins them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommitPolicy {
    /// Re-scored every tick; the holder gets momentum and exit-side thresholds.
    /// Counts as interrupt tier 0.
    Momentum,
    /// Not re-scored until its finite part finishes or the brain drops it, so
    /// its start conditions are read once: a committed motion carries no
    /// distance-range threshold that would cancel it mid-flight. Only a
    /// committed behavior with a higher `interrupt_tier` may take one of its
    /// slots, and taking either slot ends the whole behavior.
    UntilFinished { interrupt_tier: u8 },
}

impl CommitPolicy {
    /// Whether this is [`CommitPolicy::UntilFinished`].
    pub const fn is_committed(self) -> bool {
        matches!(self, Self::UntilFinished { .. })
    }

    /// The tier a holder must be outranked by to lose its slots; 0 for momentum.
    pub const fn interrupt_tier(self) -> u8 {
        match self {
            Self::UntilFinished { interrupt_tier } => interrupt_tier,
            Self::Momentum => 0,
        }
    }
}

/// A complete static brain: the list of behaviors it scores.
#[derive(Debug)]
pub struct BrainDefinition {
    pub behaviors: &'static [Behavior],
}

/// A behavior holding a slot, by its index in the definition, with its
/// interrupt tier (0 for momentum).
#[derive(Clone, Copy, Debug)]
pub struct SlotHolder {
    pub behavior: usize,
    pub interrupt_tier: u8,
}

/// Who holds each slot going into a decision.
#[derive(Clone, Copy, Debug, Default)]
pub struct SlotHolders {
    pub motion: Option<SlotHolder>,
    pub action: Option<SlotHolder>,
}

/// The behaviors that won a slot this decision, by index. A slot nothing
/// claimed is `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Assignment {
    pub motion: Option<usize>,
    pub action: Option<usize>,
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

/// One consideration's part in a score: the raw value it read (`None` when the
/// input is absent, which vetoes) and its score (`None` when the response
/// rejects the raw value).
#[derive(Clone, Copy, Debug)]
pub struct ConsiderationScoreExplanation {
    pub name: &'static str,
    pub input: ConsiderationInput,
    pub raw: Option<f32>,
    pub score: Option<f32>,
}

/// How one behavior scored in a decision, for the debug trace. `skipped` says
/// why it sat out unscored; otherwise `score` is its final score with
/// `momentum` already applied, or `None` when a consideration rejects its
/// input or the result is not a valid score.
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
    // The geometric mean over considerations: a zero anywhere still vetoes,
    // and passing flags add no penalty but raise graded inputs to a root of
    // the count, so adding a flag shifts a behavior's graded crossovers.
    // Brains are tuned against that root. No considerations scores the weight.
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

/// Fills the open slots. A committed holder is not scored, and its slots are
/// open only to committed behaviors in a higher interrupt tier; every behavior
/// whose claimed slots are all open is scored, then taken from the highest
/// score down whenever its slots are still free this decision (ties go to the
/// earlier behavior). An interrupter wins its slot by score like any other.
/// `start` is asked before a winner is taken, and a `false` drops it and moves
/// on to the next best.
pub fn assign_behaviors_to_slots(
    definition: &BrainDefinition,
    decision_context: &DecisionContext,
    availability: &Availability,
    holders: SlotHolders,
    mut start: impl FnMut(usize) -> bool,
) -> Assignment {
    let mut scores = [0.0_f32; MAX_BEHAVIORS];
    for (index, behavior) in definition.behaviors.iter().enumerate() {
        if find_skip_reason(behavior, availability, holders).is_some() {
            continue;
        }
        let is_current = holds_slot(holders, index);
        let Some(score) = score_behavior(behavior, decision_context, is_current) else {
            continue;
        };
        scores[index] = score * momentum_multiplier(is_current);
    }

    let mut assignment = Assignment::default();
    loop {
        let mut best: Option<usize> = None;
        for (index, behavior) in definition.behaviors.iter().enumerate() {
            let taken = (behavior.motion.is_some() && assignment.motion.is_some())
                || (behavior.action.is_some() && assignment.action.is_some());
            if scores[index] <= 0.0 || taken {
                continue;
            }
            if best.is_none_or(|leader| scores[index] > scores[leader]) {
                best = Some(index);
            }
        }
        let Some(index) = best else {
            return assignment;
        };
        if !start(index) {
            scores[index] = 0.0;
            continue;
        }
        let behavior = definition.behaviors[index];
        if behavior.motion.is_some() {
            assignment.motion = Some(index);
        }
        if behavior.action.is_some() {
            assignment.action = Some(index);
        }
    }
}

/// Rejects brains that compile but cannot behave: an empty or oversized list,
/// a behavior that claims nothing or has a non-finite or non-positive weight, a
/// committed behavior without exactly one finite part (one finish must end it),
/// over an idle action or a held equipment use, or in tier 0 (momentum's), or a
/// finite motion that is not committed (nothing would track its end). Intervals
/// and thresholds are checked by their constructors at compile time.
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
                "behavior {index} is committed in tier 0, which is momentum's"
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
    availability: &Availability,
    holders: SlotHolders,
) -> BehaviorScoreExplanation {
    let behavior = &definition.behaviors[index];
    let is_current = holds_slot(holders, index);
    let momentum = momentum_multiplier(is_current);
    let skipped = find_skip_reason(behavior, availability, holders).map(|reason| {
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
            raw: read_input(consideration.input, decision_context),
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

// Why a behavior sits out unscored: a slot it claims is held by a committed
// behavior it does not outrank (so a committed holder is never re-scored), or
// it may not start now.
fn find_skip_reason(
    behavior: &Behavior,
    availability: &Availability,
    holders: SlotHolders,
) -> Option<SkipReason> {
    let tier = behavior.commit.interrupt_tier();
    let is_closed = |holder: Option<SlotHolder>| {
        holder.is_some_and(|holder| holder.interrupt_tier > 0 && tier <= holder.interrupt_tier)
    };
    if (behavior.motion.is_some() && is_closed(holders.motion))
        || (behavior.action.is_some() && is_closed(holders.action))
    {
        return Some(SkipReason::SlotCommitted);
    }
    let unavailable = behavior.motion.is_some_and(|motion| !availability[motion])
        || behavior.action.is_some_and(|action| !availability[action]);
    unavailable.then_some(SkipReason::Unavailable)
}

fn holds_slot(holders: SlotHolders, index: usize) -> bool {
    [holders.motion, holders.action]
        .into_iter()
        .flatten()
        .any(|holder| holder.behavior == index)
}

fn momentum_multiplier(is_current: bool) -> f32 {
    if is_current { MOMENTUM } else { 1.0 }
}

fn score_consideration(
    consideration: &Consideration,
    decision_context: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // A missing input (no target, no anchor) vetoes, inverted or not: a fact
    // about something absent cannot be true.
    let Some(raw) = read_input(consideration.input, decision_context) else {
        return Some(0.0);
    };
    let score = consideration.response.score(raw, is_current)?;
    Some(if consideration.inverted {
        1.0 - score
    } else {
        score
    })
}
