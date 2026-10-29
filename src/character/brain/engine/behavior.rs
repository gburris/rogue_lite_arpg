//! Behaviors and static brain definitions, and the check that a definition can run.

use super::Consideration;
use crate::character::brain::{Action, Motion};

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
