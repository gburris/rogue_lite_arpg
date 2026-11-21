//! How a consideration turns one raw input into a `0.0..=1.0` score.

use super::assignment::{SkipReason, find_skip_reason, momentum_multiplier};
use super::behavior::{Behavior, BrainDefinition, BrainVocabulary};
use crate::interval::Interval;

/// One normalized input to a behavior's score.
#[derive(Clone, Copy, Debug)]
pub struct Consideration<V: BrainVocabulary> {
    pub(in crate::character::brain) name: &'static str,
    pub(in crate::character::brain) input: V::Input,
    pub(in crate::character::brain) score_mapping: ScoreMapping,
    pub(in crate::character::brain) inverted: bool,
}

impl<V: BrainVocabulary> Consideration<V> {
    /// The opposite fact under a new name. A missing input still vetoes.
    pub const fn inverted(self, name: &'static str) -> Self {
        Self {
            name,
            inverted: !self.inverted,
            ..self
        }
    }
}

/// How a consideration turns its raw input into a `0.0..=1.0` score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::character::brain) enum ScoreMapping {
    /// Normalize over `range`, then shape with `curve`.
    Curve { range: Interval, curve: Curve },
    /// Full once the raw input crosses `enter`, zero before it. While its
    /// behavior holds a slot the threshold is `exit` instead, so the gap keeps
    /// the behavior. `exit < enter` passes raw inputs at or above the threshold;
    /// otherwise, including no gap at all, it passes raw inputs at or below it.
    Threshold { enter: f32, exit: f32 },
}

impl ScoreMapping {
    /// Score for `raw`, or `None` for a non-finite raw input.
    pub(super) fn score_raw_input(self, raw: f32, is_current: bool) -> Option<f32> {
        match self {
            Self::Curve { range, curve } => {
                range.normalize_clamped(raw).map(|value| curve.apply(value))
            }
            Self::Threshold { enter, exit } => {
                if !raw.is_finite() {
                    return None;
                }
                let threshold = if is_current { exit } else { enter };
                let passes = if exit < enter {
                    raw >= threshold
                } else {
                    raw <= threshold
                };
                Some(if passes { 1.0 } else { 0.0 })
            }
        }
    }
}

/// Curve applied after normalizing a consideration's raw value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Linear,
    /// Steepens a partial input (`x^3`).
    Cubic,
}

impl Curve {
    fn apply(self, normalized: f32) -> f32 {
        let value = match self {
            Self::Linear => normalized,
            Self::Cubic => normalized * normalized * normalized,
        };

        value.clamp(0.0, 1.0)
    }
}

/// Whether one of `behavior`'s considerations reads an input that is now absent, such as a
/// target distance with no target.
pub(in crate::character::brain) fn has_absent_input<V: BrainVocabulary>(
    behavior: &Behavior<V>,
    read: &impl Fn(V::Input) -> Option<f32>,
) -> bool {
    behavior
        .considerations
        .iter()
        .any(|consideration| read(consideration.input).is_none())
}

/// Final bounded score for one behavior, reading each consideration's input
/// through `read`. `is_current` switches threshold considerations to their exit side.
pub(super) fn score_behavior<V: BrainVocabulary>(
    behavior: &Behavior<V>,
    read: &impl Fn(V::Input) -> Option<f32>,
    is_current: bool,
) -> Option<f32> {
    combine_consideration_scores(
        behavior.weight,
        behavior.considerations.iter().map(|consideration| {
            score_consideration(consideration, read(consideration.input), is_current)
        }),
    )
}

/// Weight times the geometric mean of `scores`; `None` when a score before any zero is `None`.
/// The first zero vetoes and stops there, leaving later scores unread, so a behavior lists its
/// cheap considerations first. Each passing flag raises graded inputs to a higher root, so adding
/// a flag shifts a behavior's crossovers. Brains are tuned against that root.
pub(super) fn combine_consideration_scores(
    weight: f32,
    scores: impl IntoIterator<Item = Option<f32>>,
) -> Option<f32> {
    let mut product = 1.0;
    let mut count = 0;
    for score in scores {
        let score = score?;
        if score == 0.0 {
            return Some(0.0);
        }
        product *= score;
        count += 1;
    }
    let mut score = weight;
    if count > 0 {
        score *= product.powf(1.0 / (count as f32));
    }

    (score.is_finite() && score >= 0.0).then_some(score)
}

/// Scores one consideration from its `raw` input; `None` when the score
/// mapping rejects a non-finite raw value.
pub(super) fn score_consideration<V: BrainVocabulary>(
    consideration: &Consideration<V>,
    raw: Option<f32>,
    is_current: bool,
) -> Option<f32> {
    // A missing input (no target, no home area) vetoes, inverted or not.
    let Some(raw) = raw else {
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

/// One consideration's part in a score. `raw` is `None` when the input is
/// absent; `score` is `None` when the score mapping rejects the raw value.
#[derive(Clone, Copy, Debug)]
pub struct ConsiderationScoreExplanation<V: BrainVocabulary> {
    pub name: &'static str,
    pub input: V::Input,
    pub raw: Option<f32>,
    pub score: Option<f32>,
}

/// How one behavior scored, for the debug trace: why it sat out, or its final
/// score with `momentum` applied (`None` when invalid).
#[derive(Clone, Debug)]
pub struct BehaviorScoreExplanation<V: BrainVocabulary> {
    pub skipped: Option<SkipReason>,
    pub score: Option<f32>,
    pub momentum: f32,
    pub considerations: Vec<ConsiderationScoreExplanation<V>>,
}

/// Explains behavior `index` exactly as
/// [`assign_behaviors_to_slots`](super::assignment::assign_behaviors_to_slots) scores it from the
/// same inputs, reading each consideration's input once.
pub(in crate::character::brain) fn explain_behavior<V: BrainVocabulary>(
    definition: &BrainDefinition<V>,
    index: usize,
    read: &impl Fn(V::Input) -> Option<f32>,
    available: &impl Fn(usize) -> bool,
    running_behavior_ids: &[usize],
) -> BehaviorScoreExplanation<V> {
    let behavior = &definition.behaviors[index];
    let is_current = running_behavior_ids.contains(&index);
    let momentum = momentum_multiplier(is_current);
    let skipped = find_skip_reason(definition, index, available, running_behavior_ids);
    if skipped.is_some() {
        return BehaviorScoreExplanation {
            skipped,
            score: None,
            momentum,
            considerations: Vec::new(),
        };
    }
    let considerations: Vec<_> = behavior
        .considerations
        .iter()
        .map(|consideration| {
            let raw = read(consideration.input);
            ConsiderationScoreExplanation {
                name: consideration.name,
                input: consideration.input,
                raw,
                score: score_consideration(consideration, raw, is_current),
            }
        })
        .collect();
    BehaviorScoreExplanation {
        skipped,
        score: combine_consideration_scores(
            behavior.weight,
            considerations.iter().map(|explanation| explanation.score),
        )
        .map(|score| score * momentum),
        momentum,
        considerations,
    }
}
