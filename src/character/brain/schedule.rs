use bevy::prelude::*;

/// Ordered AI phases. Perception feeds targeting, aim, decisions, execution
/// of those decisions, and finally the motion driver.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum AiSet {
    Perception,
    Targeting,
    Aim,
    Decision,
    Execute,
    Motion,
}
