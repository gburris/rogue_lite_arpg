//! Debug explanations of how a behavior scored, computed from the same inputs as
//! slot assignment.

use super::assignment::{SkipReason, find_skip_reason, momentum_multiplier};
use super::behavior::{BrainDefinition, BrainVocabulary};
use super::scoring::{combine_consideration_scores, score_consideration};

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
