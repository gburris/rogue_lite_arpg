use super::MotionKey;
use super::engine::{self, BrainVocabulary, CommitPolicy, Curve, ScoreMapping};
use super::inputs::{ConsiderationInput, DecisionInputs};
use crate::interval::Interval;
use crate::prelude::*;

/// The motions, actions and inputs of character brains, for the scoring engine.
#[derive(Clone, Copy, Debug)]
pub struct CharacterVocabulary;

impl BrainVocabulary for CharacterVocabulary {
    type Motion = MotionKey;
    type Action = Action;
    type Input = ConsiderationInput;

    /// Idling never reports a finish. A sustained use doesn't either, which depends on the
    /// character's source; see [`Behavior::suits_commit_policy`].
    fn action_reports_finish(action: Action) -> bool {
        match action {
            Action::Idle => false,
            Action::Use(_) => true,
        }
    }
}

/// A behavior of a character brain.
pub type Behavior = engine::Behavior<CharacterVocabulary>;

impl Behavior {
    /// Whether the deciding character's sources for this behavior's motion and action end on
    /// their own in the way its commit policy needs: a committed behavior needs exactly one part
    /// that does, since nothing else would end it, and a continuously scored one needs a motion
    /// that runs until replaced.
    pub(super) fn suits_commit_policy(&self, inputs: &DecisionInputs) -> bool {
        let finite_motion = self
            .motion
            .is_some_and(|key| inputs.motion_ends_on_its_own(key));
        if !self.commit.is_committed() {
            return !finite_motion;
        }
        let finite_action = self
            .action
            .is_some_and(|action| inputs.action_ends_on_its_own(action));
        usize::from(finite_motion) + usize::from(finite_action) == 1
    }
}

/// A consideration of a character brain.
pub type Consideration = engine::Consideration<CharacterVocabulary>;
/// A character brain's static definition.
pub type BrainDefinition = engine::BrainDefinition<CharacterVocabulary>;
/// How a behavior of a character brain scored.
pub type BehaviorScoreExplanation = engine::BehaviorScoreExplanation<CharacterVocabulary>;

/// The one non-movement thing a character does at a time: use what it has in a slot, or
/// deliberately do nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Use the item equipped in a slot or one of the character's abilities.
    Use(ActionKey),
    /// Chose to do nothing, unlike an empty action slot where nothing scored.
    /// Always available; no applier carries it out.
    Idle,
}

impl Action {
    /// The key of the action source this action uses; `None` for `Idle`.
    pub fn key(self) -> Option<ActionKey> {
        match self {
            Self::Use(key) => Some(key),
            Self::Idle => None,
        }
    }
}

/// Preferred minimum distance a ranged or flying brain keeps from its target.
pub(in crate::character) const CAST_MIN_RANGE: f32 = 200.0;
/// Preferred maximum distance a ranged or flying brain keeps from its target.
pub(in crate::character) const CAST_MAX_RANGE: f32 = 360.0;
pub(in crate::character) const CAST_DISTANCE_RANGE: Interval =
    Interval::new(CAST_MIN_RANGE, CAST_MAX_RANGE);

pub(in crate::character) const CAST_HYSTERESIS: f32 = 5.0;

pub(in crate::character) const MAINHAND: Action = Action::Use(ActionKey::MAINHAND);

/// Maps `input` over `range` through `curve`.
pub const fn consideration(
    name: &'static str,
    input: ConsiderationInput,
    range: Interval,
    curve: Curve,
) -> Consideration {
    Consideration {
        name,
        input,
        score_mapping: ScoreMapping::Curve { range, curve },
        inverted: false,
    }
}

/// Whether [`ConsiderationInput::SecondsMotionHeld`] for `motion` exceeds `seconds`.
pub(in crate::character) const fn motion_held_for(
    name: &'static str,
    motion: MotionKey,
    seconds: f32,
) -> Consideration {
    Consideration {
        name,
        input: ConsiderationInput::SecondsMotionHeld(motion),
        score_mapping: ScoreMapping::Threshold {
            enter: seconds,
            exit: seconds,
        },
        inverted: true,
    }
}

/// A target-distance threshold with hysteresis; see [`ScoreMapping::Threshold`].
pub(in crate::character) const fn distance_threshold(
    name: &'static str,
    enter: f32,
    exit: f32,
) -> Consideration {
    assert!(
        enter.is_finite() && exit.is_finite(),
        "threshold needs finite enter/exit"
    );
    Consideration {
        name,
        input: ConsiderationInput::TargetDistance,
        score_mapping: ScoreMapping::Threshold { enter, exit },
        inverted: false,
    }
}

/// Mana fraction at or below `enter`, held up to `exit` while its behavior is
/// current; see [`ScoreMapping::Threshold`].
pub(in crate::character) const fn mana_at_or_below(
    name: &'static str,
    enter: f32,
    exit: f32,
) -> Consideration {
    assert!(
        0.0 <= enter && enter <= exit && exit <= 1.0,
        "mana threshold needs 0 <= enter <= exit <= 1"
    );
    Consideration {
        name,
        input: ConsiderationInput::ManaFraction,
        score_mapping: ScoreMapping::Threshold { enter, exit },
        inverted: false,
    }
}

/// A reach-fraction threshold with hysteresis on `input`, one of the reach
/// fraction inputs; see [`ScoreMapping::Threshold`].
pub(in crate::character) const fn reach_fraction_threshold(
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
        score_mapping: ScoreMapping::Threshold { enter, exit },
        inverted: false,
    }
}

/// Target within the reach `input` reads (a step at a fraction of 1.0).
pub(in crate::character) const fn within_reach(
    name: &'static str,
    input: ConsiderationInput,
) -> Consideration {
    reach_fraction_threshold(name, input, 1.0, 1.0)
}

/// Target at most `distance` away, with no hysteresis.
pub(in crate::character) const fn target_within_distance(
    name: &'static str,
    distance: f32,
) -> Consideration {
    distance_threshold(name, distance, distance)
}

/// Target moving at most `speed`, with no hysteresis.
pub(in crate::character) const fn target_speed_at_most(
    name: &'static str,
    speed: f32,
) -> Consideration {
    assert!(speed.is_finite(), "threshold needs a finite speed");
    Consideration {
        name,
        input: ConsiderationInput::TargetSpeed,
        score_mapping: ScoreMapping::Threshold {
            enter: speed,
            exit: speed,
        },
        inverted: false,
    }
}

/// A 0-or-1 input scored as itself.
pub(in crate::character) const fn flag(
    name: &'static str,
    input: ConsiderationInput,
) -> Consideration {
    consideration(name, input, Interval::new(0.0, 1.0), Curve::Linear)
}

/// A 0-or-1 input that must be 1 to start its behavior, and passes while the behavior holds its
/// slot.
pub(in crate::character) const fn flag_kept_while_current(
    name: &'static str,
    input: ConsiderationInput,
) -> Consideration {
    Consideration {
        name,
        input,
        score_mapping: ScoreMapping::Threshold {
            enter: 1.0,
            exit: -1.0,
        },
        inverted: false,
    }
}

pub(in crate::character) const HAS_TARGET: Consideration =
    flag("has target", ConsiderationInput::HasTarget);
pub(in crate::character) const NO_TARGET: Consideration = HAS_TARGET.inverted("no target");
pub(in crate::character) const LINE_OF_SIGHT: Consideration =
    flag("line of sight", ConsiderationInput::LineOfSight);
const LOST_SIGHT: Consideration = LINE_OF_SIGHT.inverted("lost sight");
const HOME_AREA_PULL: Consideration = consideration(
    "home area pull",
    ConsiderationInput::HomeAreaPull,
    Interval::new(0.0, 1.0),
    Curve::Cubic,
);
pub(in crate::character) const ATTACK_FINISHED: Consideration =
    flag("attack finished", ConsiderationInput::AttackFinished);
pub(in crate::character) const IN_MAINHAND_REACH: Consideration = within_reach(
    "in mainhand reach",
    ConsiderationInput::ActionReachFraction(ActionKey::MAINHAND),
);

// Both enter at one shared distance, so a character with no incumbent always
// has one of the two; the wide exits let an incumbent hold on.
const BEYOND_CAST_DISTANCE_RANGE: Consideration = distance_threshold(
    "beyond cast distance range",
    CAST_MAX_RANGE,
    CAST_MAX_RANGE - CAST_HYSTERESIS,
);
pub(in crate::character) const INSIDE_OUTER_CAST_EDGE: Consideration = distance_threshold(
    "inside outer cast edge",
    CAST_MAX_RANGE,
    CAST_MAX_RANGE + CAST_HYSTERESIS,
);

pub(in crate::character) const WANDER: Behavior = Behavior::new(
    Some(MotionKey::Wander),
    None,
    CommitPolicy::ContinuouslyScored,
    1.0,
    &[NO_TARGET],
);
pub(in crate::character) const RETURN_TO_HOME: Behavior = Behavior::new(
    Some(MotionKey::ReturnToHome),
    None,
    CommitPolicy::ContinuouslyScored,
    6.0,
    &[NO_TARGET, HOME_AREA_PULL],
);
pub(in crate::character) const CHASE_CAST_RANGE: Behavior = Behavior::new(
    Some(MotionKey::Chase),
    None,
    CommitPolicy::ContinuouslyScored,
    4.0,
    &[BEYOND_CAST_DISTANCE_RANGE],
);

/// Walks to where a target that slipped out of sight was last seen.
pub(in crate::character) const SEEK_LOST_TARGET: Behavior = Behavior::new(
    Some(MotionKey::Chase),
    None,
    CommitPolicy::ContinuouslyScored,
    7.0,
    &[LOST_SIGHT],
);

/// Doing nothing on the action slot, at a low constant weight, for brains whose
/// actions all ride in two-slot behaviors that outweigh it anyway. A brain with
/// action-only behaviors sets its own baseline as the bar they must beat.
pub(in crate::character) const ACTION_BASELINE_IDLE: Behavior = Behavior::new(
    None,
    Some(Action::Idle),
    CommitPolicy::ContinuouslyScored,
    0.01,
    &[],
);
