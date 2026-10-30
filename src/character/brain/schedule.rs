use bevy::prelude::*;

/// Ordered AI phases. Perception feeds targeting, aim, decisions, execution
/// of those decisions, and finally the motion driver.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AiSet {
    Perception,
    Targeting,
    Aim,
    Decision,
    Execute,
    Motion,
}
