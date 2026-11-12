//! The pure utility scoring engine: behaviors, static brain definitions, and
//! the math that fills the motion and action slots. No ECS and no content.

mod assignment;
mod behavior;
mod explanation;
mod scoring;

pub(super) use self::assignment::assign_behaviors_to_slots;
pub(super) use self::behavior::validate_brain;
pub use self::behavior::{Behavior, BrainDefinition, CommitPolicy};
pub use self::explanation::BehaviorScoreExplanation;
pub(super) use self::explanation::explain_behavior;
pub(super) use self::scoring::ScoreMapping;
pub use self::scoring::{Consideration, Curve, Interval};
