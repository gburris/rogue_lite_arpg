//! The pure utility scoring engine: behaviors, static brain definitions, and
//! the math that fills the motion and action slots. No ECS and no content.

mod assignment;
mod behavior;
mod scoring;

pub(super) use self::assignment::assign_behaviors_to_slots;
pub(super) use self::behavior::validate_brain;
pub use self::behavior::{Behavior, BrainDefinition, BrainVocabulary, CommitPolicy};
pub use self::scoring::{BehaviorScoreExplanation, Consideration, Curve};
pub(super) use self::scoring::{ScoreMapping, explain_behavior, has_absent_input};
