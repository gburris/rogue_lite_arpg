use crate::character::brain::considerations::{ACTION_BASELINE_IDLE, RETURN_TO_HOME, WANDER};
use crate::character::brain::{Behavior, BrainDefinition};

static VILLAGER_BEHAVIORS: &[Behavior] = &[WANDER, ACTION_BASELINE_IDLE, RETURN_TO_HOME];

/// Wanders around its home area and never requests equipment.
pub(in crate::character) static VILLAGER: BrainDefinition = BrainDefinition {
    behaviors: VILLAGER_BEHAVIORS,
};
