//! Behaviors and static brain definitions, and the check that a definition can run.

use std::fmt::Debug;

use super::Consideration;

/// Most behaviors one brain definition may list, so scoring needs no allocation.
pub(super) const MAX_BEHAVIORS: usize = 16;

/// The game's types for what behaviors hold: their motions, actions and consideration inputs. The
/// engine only copies and compares them, and asks which actions report a finish. Any motion may
/// report one.
pub trait BrainVocabulary: Clone + Copy + Debug + 'static {
    type Motion: Clone + Copy + Debug + 'static;
    type Action: Clone + Copy + Debug + 'static;
    type Input: Clone + Copy + Debug + 'static;

    /// Whether `action` can end on its own and report it, so its finish can end a committed
    /// behavior.
    fn action_reports_finish(action: Self::Action) -> bool;
}

/// One candidate a brain can choose: a motion, an action, or both, and how
/// firmly it holds them.
#[derive(Clone, Copy, Debug)]
pub struct Behavior<V: BrainVocabulary> {
    pub motion: Option<V::Motion>,
    pub action: Option<V::Action>,
    pub(in crate::character::brain) commit: CommitPolicy,
    pub(in crate::character::brain) weight: f32,
    pub(in crate::character::brain) considerations: &'static [Consideration<V>],
    pub(in crate::character::brain) start_chance: Option<StartChance>,
}

impl<V: BrainVocabulary> Behavior<V> {
    /// Builds a behavior; [`validate_brain`] checks its shape and weight.
    pub const fn new(
        motion: Option<V::Motion>,
        action: Option<V::Action>,
        commit: CommitPolicy,
        weight: f32,
        considerations: &'static [Consideration<V>],
    ) -> Self {
        Self {
            motion,
            action,
            commit,
            weight,
            considerations,
            start_chance: None,
        }
    }

    /// When this behavior wins, it starts only with probability `chance`. On a miss, the next best
    /// behavior takes the slot and this one cannot win for `retry_seconds`.
    pub const fn with_start_chance(self, chance: f32, retry_seconds: f32) -> Self {
        Self {
            start_chance: Some(StartChance {
                chance,
                retry_seconds,
            }),
            ..self
        }
    }

    /// Whether both behaviors claim the motion slot or both claim the action slot.
    pub(in crate::character::brain) fn shares_a_slot_with(&self, other: &Self) -> bool {
        (self.motion.is_some() && other.motion.is_some())
            || (self.action.is_some() && other.action.is_some())
    }
}

/// The roll a behavior makes when it wins; see [`Behavior::with_start_chance`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartChance {
    pub chance: f32,
    pub retry_seconds: f32,
}

/// How firmly a behavior holds its slots once it wins them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommitPolicy {
    /// Re-scored every tick with momentum and exit-side thresholds. Tier 0.
    ContinuouslyScored,
    /// Not re-scored until its finite part finishes or the brain drops it as stale, so its start
    /// conditions are read once. Only a committed behavior with a higher `interrupt_tier` may
    /// take one of its slots, which ends the whole behavior.
    UntilFinished { interrupt_tier: u8 },
}

impl CommitPolicy {
    pub(in crate::character::brain) const fn is_committed(self) -> bool {
        matches!(self, Self::UntilFinished { .. })
    }

    /// The tier a holder must be outranked by to lose its slots; 0 for a continuously scored
    /// behavior.
    pub(super) const fn interrupt_tier(self) -> u8 {
        match self {
            Self::UntilFinished { interrupt_tier } => interrupt_tier,
            Self::ContinuouslyScored => 0,
        }
    }
}

/// A complete static brain: the list of behaviors it scores.
#[derive(Debug)]
pub struct BrainDefinition<V: BrainVocabulary> {
    pub behaviors: &'static [Behavior<V>],
}

/// Rejects brains that compile but cannot behave. Intervals and thresholds
/// are checked by their constructors at compile time.
pub(in crate::character::brain) fn validate_brain<V: BrainVocabulary>(
    brain: &BrainDefinition<V>,
) -> Result<(), String> {
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
        if committed && behavior.commit.interrupt_tier() == 0 {
            return Err(format!(
                "behavior {index} is committed in tier 0, which is for continuously scored \
                 behaviors"
            ));
        }
        let can_finish =
            behavior.motion.is_some() || behavior.action.is_some_and(V::action_reports_finish);
        if committed && !can_finish {
            return Err(format!(
                "behavior {index} is committed with no part that can finish; a finish must end it"
            ));
        }
        if let Some(StartChance {
            chance,
            retry_seconds,
        }) = behavior.start_chance
            && !(chance > 0.0 && chance <= 1.0 && retry_seconds.is_finite() && retry_seconds >= 0.0)
        {
            return Err(format!(
                "behavior {index} start chance {chance} must be in (0, 1] and its retry \
                 {retry_seconds} s finite and non-negative"
            ));
        }
    }
    Ok(())
}
