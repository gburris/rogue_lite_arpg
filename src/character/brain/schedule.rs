use bevy::prelude::*;

/// Ordered brain phases. Perception feeds targeting, aim, decisions, execution
/// of those decisions, and finally the motion driver.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum BrainSystems {
    Perception,
    Targeting,
    Aim,
    Decision,
    Execute,
    Motion,
}
