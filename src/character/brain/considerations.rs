//! Scored facts for the brains, built over the consideration inputs in
//! [`super::perception`]. An input reports what it knows, always: `None` means
//! the input itself is absent (no target, no anchor), which vetoes the
//! decision. A consideration that reads the target therefore implies one.
//!
//! This holds the vocabulary every brain shares; each enemy's own
//! considerations live next to its brain in `character/enemy`.

use super::action::Action;
use super::engine::{Behavior, CommitPolicy, Consideration, Curve, Interval, Response};
use super::motion::Motion;
use super::perception::ConsiderationInput;
use crate::items::prelude::EquipmentSlot;

/// Preferred minimum distance a ranged or flying brain keeps from its target.
pub const CAST_MIN_RANGE: f32 = 200.0;
/// Preferred maximum distance a ranged or flying brain keeps from its target.
pub const CAST_MAX_RANGE: f32 = 360.0;
pub const CAST_DISTANCE_RANGE: Interval = Interval::new(CAST_MIN_RANGE, CAST_MAX_RANGE);

pub(crate) const CAST_HYSTERESIS: f32 = 5.0;

pub(crate) const MAINHAND: Action = Action::Use(EquipmentSlot::Mainhand);

/// Maps `input` over `range` through `curve`.
pub(crate) const fn consideration(
    name: &'static str,
    input: ConsiderationInput,
    range: Interval,
    curve: Curve,
) -> Consideration {
    Consideration {
        name,
        input,
        response: Response::Curve { range, curve },
        inverted: false,
    }
}

/// A target-distance threshold with hysteresis; see [`Response::Threshold`].
pub(crate) const fn distance_threshold(name: &'static str, enter: f32, exit: f32) -> Consideration {
    assert!(
        enter.is_finite() && exit.is_finite(),
        "threshold needs finite enter/exit"
    );
    Consideration {
        name,
        input: ConsiderationInput::TargetDistance,
        response: Response::Threshold { enter, exit },
        inverted: false,
    }
}

/// Mana fraction at or below `enter`, held up to `exit` while its behavior is
/// current; see [`Response::Threshold`].
pub(crate) const fn mana_at_or_below(name: &'static str, enter: f32, exit: f32) -> Consideration {
    assert!(
        0.0 <= enter && enter <= exit && exit <= 1.0,
        "mana threshold needs 0 <= enter <= exit <= 1"
    );
    Consideration {
        name,
        input: ConsiderationInput::ManaFraction,
        response: Response::Threshold { enter, exit },
        inverted: false,
    }
}

/// A reach-fraction threshold with hysteresis on `input`, one of the reach
/// fraction inputs; see [`Response::Threshold`].
pub(crate) const fn reach_fraction_threshold(
    name: &'static str,
    input: ConsiderationInput,
    enter: f32,
    exit: f32,
) -> Consideration {
    assert!(
        enter.is_finite() && exit.is_finite(),
        "threshold needs finite enter/exit"
    );
    Consideration {
        name,
        input,
        response: Response::Threshold { enter, exit },
        inverted: false,
    }
}

/// Target within the reach `input` reads (a step at a fraction of 1.0).
pub(crate) const fn within_reach(name: &'static str, input: ConsiderationInput) -> Consideration {
    reach_fraction_threshold(name, input, 1.0, 1.0)
}

/// Target at most `distance` away, with no hysteresis.
pub(crate) const fn target_within_distance(name: &'static str, distance: f32) -> Consideration {
    distance_threshold(name, distance, distance)
}

/// A 0-or-1 input scored as itself.
pub(crate) const fn flag(name: &'static str, input: ConsiderationInput) -> Consideration {
    consideration(name, input, Interval::new(0.0, 1.0), Curve::Linear)
}

pub(crate) const HAS_TARGET: Consideration = flag("has target", ConsiderationInput::HasTarget);
pub(crate) const NO_TARGET: Consideration = HAS_TARGET.inverted("no target");
pub(crate) const LINE_OF_SIGHT: Consideration =
    flag("line of sight", ConsiderationInput::LineOfSight);
pub(crate) const ANCHOR_PULL: Consideration = consideration(
    "anchor pull",
    ConsiderationInput::AnchorPull,
    Interval::new(0.0, 1.0),
    Curve::Cubic,
);
pub(crate) const ATTACK_FINISHED: Consideration =
    flag("attack finished", ConsiderationInput::AttackFinished);
pub(crate) const IN_MAINHAND_REACH: Consideration = within_reach(
    "in mainhand reach",
    ConsiderationInput::ActionReachFraction(MAINHAND),
);

// The pair below enters at one shared distance, so a character with no incumbent
// always has one of the two; the exits stay wide so an incumbent holds on.
pub(crate) const BEYOND_CAST_DISTANCE_RANGE: Consideration = distance_threshold(
    "beyond cast distance range",
    CAST_MAX_RANGE,
    CAST_MAX_RANGE - CAST_HYSTERESIS,
);
pub(crate) const INSIDE_OUTER_CAST_EDGE: Consideration = distance_threshold(
    "inside outer cast edge",
    CAST_MAX_RANGE,
    CAST_MAX_RANGE + CAST_HYSTERESIS,
);

/// Disengaged ambling; each character's `Wander` component sets the rhythm.
pub(crate) const CALM: Behavior = Behavior::new(
    Some(Motion::Calm),
    None,
    CommitPolicy::Momentum,
    1.0,
    &[NO_TARGET],
);
pub(crate) const RETURN_TO_ANCHOR: Behavior = Behavior::new(
    Some(Motion::ReturnToAnchor),
    None,
    CommitPolicy::Momentum,
    6.0,
    &[NO_TARGET, ANCHOR_PULL],
);
pub(crate) const CHASE_CAST_RANGE: Behavior = Behavior::new(
    Some(Motion::Chase),
    None,
    CommitPolicy::Momentum,
    4.0,
    &[BEYOND_CAST_DISTANCE_RANGE],
);

/// Doing nothing on the action slot, at a low constant weight, for brains whose
/// actions all ride in two-slot behaviors that outweigh it anyway. A brain with
/// action-only behaviors sets its own baseline as the bar they must beat.
pub(crate) const ACTION_BASELINE_IDLE: Behavior =
    Behavior::new(None, Some(Action::Idle), CommitPolicy::Momentum, 0.01, &[]);
