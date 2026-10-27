//! Helpers for scoring tests that pick a brain's motion and action from a
//! hand-built context.

use bevy::prelude::Vec2;

use super::action::Action;
use super::capability::{Availability, MotionActionTable};
use super::considerations::*;
use super::engine::{BrainDefinition, SlotHolder, SlotHolders, assign_behaviors_to_slots};
use super::motion::Motion;
pub(crate) use super::perception::{AnchorView, DecisionContext, SelfView, TargetView};

fn context() -> DecisionContext {
    DecisionContext::default()
}

pub(crate) fn visible_target(distance: f32) -> TargetView {
    TargetView {
        distance,
        line_of_sight: true,
        bearing: Vec2::X,
        velocity: Vec2::ZERO,
        seconds_since_targeted: 0.0,
    }
}

/// Every continuous motion and action, and no finite motion, as a character
/// carrying `Wander` and `Orbit` but no finite-motion component reports.
pub(crate) fn available() -> Availability {
    let mut table = MotionActionTable::splat(true);
    for &motion in Motion::ALL.iter().filter(|motion| motion.is_finite()) {
        table[motion] = false;
    }
    Availability(table)
}

/// A target against a mainhand item with `reach`, as perception observes it.
pub(crate) fn armed(distance: f32, reach: f32) -> DecisionContext {
    let mut decision_context = DecisionContext {
        target: Some(visible_target(distance)),
        ..context()
    };
    decision_context.reach_fraction = MotionActionTable::splat(Some(f32::MAX));
    decision_context.reach_fraction[MAINHAND] = Some(distance / reach);
    decision_context
}

/// The motion and action the brain picks while `current` holds the motion slot.
pub(crate) fn choose(
    brain: &BrainDefinition,
    decision_context: &DecisionContext,
    current: Motion,
) -> (Motion, Option<Action>) {
    choose_with(brain, decision_context, current, &available())
}

pub(crate) fn choose_with(
    brain: &BrainDefinition,
    decision_context: &DecisionContext,
    current: Motion,
    availability: &Availability,
) -> (Motion, Option<Action>) {
    let holder = brain
        .behaviors
        .iter()
        .position(|behavior| behavior.motion == Some(current))
        .expect("brain lists the current motion");
    let holders = SlotHolders {
        motion: Some(SlotHolder {
            behavior: holder,
            interrupt_tier: 0,
        }),
        action: None,
    };
    let assignment =
        assign_behaviors_to_slots(brain, decision_context, availability, holders, |_| true);
    let motion = assignment
        .motion
        .and_then(|index| brain.behaviors[index].motion)
        .expect("a motion scores");
    let action = assignment
        .action
        .and_then(|index| brain.behaviors[index].action);
    (motion, action)
}
