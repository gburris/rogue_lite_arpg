//! The pure utility scoring engine: behaviors, static brain definitions, and
//! the math that fills the motion and action slots. No ECS and no content.

mod assignment;
mod behavior;
mod explanation;
mod scoring;

pub use self::assignment::assign_behaviors_to_slots;
pub use self::behavior::{Behavior, BrainDefinition, CommitPolicy, validate_brain};
pub use self::explanation::{BehaviorScoreExplanation, explain_behavior};
pub use self::scoring::{Consideration, Curve, Interval, ScoreMapping};
