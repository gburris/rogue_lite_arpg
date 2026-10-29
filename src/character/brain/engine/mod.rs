//! The pure utility scoring engine: behaviors, static brain definitions, and
//! the math that fills the motion and action slots. No ECS and no content.

mod assignment;
mod behavior;
mod explanation;
mod scoring;

pub use assignment::assign_behaviors_to_slots;
pub use behavior::{Behavior, BrainDefinition, CommitPolicy, validate_brain};
pub use explanation::{BehaviorScoreExplanation, explain_behavior};
pub use scoring::{Consideration, Curve, Interval, ScoreMapping};
