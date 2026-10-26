//! Input extractors and scored facts for the brains. Extractors report what they
//! know, always: `None` means the input itself is absent (no target, no
//! anchor), which vetoes the decision. A consideration that extracts from the target
//! therefore implies one.

use super::model::{
    Ability, Action, CAST_DISTANCE_RANGE, CAST_MAX_RANGE, CAST_MIN_RANGE, Consideration, Curve,
    DecisionContext, HEAL_SAFE_DISTANCE, Interval, MELEE_RANGE, Motion, Response,
    action_table_index, motion_table_index,
};
use crate::items::prelude::EquipmentSlot;

pub(crate) const MELEE_HYSTERESIS: f32 = 4.0;
/// Melee reach including the hold threshold's hysteresis: where a parked warrior can still swing.
pub(crate) const MELEE_REACH: f32 = MELEE_RANGE + MELEE_HYSTERESIS;
const CAST_HYSTERESIS: f32 = 5.0;

fn normalize_boolean(set: bool) -> Option<f32> {
    Some(if set { 1.0 } else { 0.0 })
}

fn extract_has_target(decision_context: &DecisionContext) -> Option<f32> {
    normalize_boolean(decision_context.target.is_some())
}

fn extract_line_of_sight(decision_context: &DecisionContext) -> Option<f32> {
    decision_context
        .target
        .map(|target| if target.line_of_sight { 1.0 } else { 0.0 })
}

/// How far past its anchor range the body is. Once already heading home it
/// reads full until arrival, so crossing back inside the range mid-return
/// cannot hand the body straight back to calm.
fn extract_anchor_pull(decision_context: &DecisionContext) -> Option<f32> {
    let anchor = decision_context.me.anchor?;
    if decision_context.memory.motion == Some(Motion::ReturnToAnchor) {
        Some(if anchor.arrived { 0.0 } else { 1.0 })
    } else {
        Some(anchor.overshoot_fraction)
    }
}

fn extract_missing_health(decision_context: &DecisionContext) -> Option<f32> {
    Some(1.0 - decision_context.me.health_fraction)
}

fn extract_target_distance(decision_context: &DecisionContext) -> Option<f32> {
    decision_context.target.map(|target| target.distance)
}

/// Whether the motion at index `MOTION` is the one running.
fn extract_motion_is_running<const MOTION: usize>(
    decision_context: &DecisionContext,
) -> Option<f32> {
    normalize_boolean(
        decision_context
            .memory
            .motion
            .is_some_and(|motion| motion.index() == MOTION),
    )
}

fn extract_attack_finished(decision_context: &DecisionContext) -> Option<f32> {
    normalize_boolean(decision_context.memory.attack_finished)
}

fn extract_circling(decision_context: &DecisionContext) -> Option<f32> {
    normalize_boolean(matches!(
        decision_context.memory.motion,
        Some(Motion::Orbit(_) | Motion::KeepDistance(_))
    ))
}

/// Whether the target is inside the reach of the capability at table index `INDEX`.
fn extract_in_reach<const INDEX: usize>(decision_context: &DecisionContext) -> Option<f32> {
    normalize_boolean(decision_context.in_reach[INDEX])
}

fn extract_recovering(decision_context: &DecisionContext) -> Option<f32> {
    // A mana break is only meaningful while the pool is still short; the
    // held state supplies the hysteresis and the fraction keeps the sense
    // honest if a caster ever starts a break from a full pool.
    normalize_boolean(
        decision_context.me.mana_recovering && decision_context.me.mana_fraction < 1.0,
    )
}

const fn consideration(
    name: &'static str,
    extract_raw_input: fn(&DecisionContext) -> Option<f32>,
    range: Interval,
    curve: Curve,
) -> Consideration {
    Consideration {
        name,
        extract_raw_input,
        response: Response::Curve { range, curve },
        inverted: false,
    }
}

/// A target-distance threshold with hysteresis; see [`Response::Threshold`].
const fn distance_threshold(name: &'static str, enter: f32, exit: f32) -> Consideration {
    assert!(
        enter.is_finite() && exit.is_finite(),
        "threshold needs finite enter/exit"
    );
    Consideration {
        name,
        extract_raw_input: extract_target_distance,
        response: Response::Threshold { enter, exit },
        inverted: false,
    }
}

/// Target at most `distance` away, with no hysteresis.
const fn target_within_distance(name: &'static str, distance: f32) -> Consideration {
    distance_threshold(name, distance, distance)
}

const fn flag(
    name: &'static str,
    extract_raw_input: fn(&DecisionContext) -> Option<f32>,
) -> Consideration {
    consideration(
        name,
        extract_raw_input,
        Interval::new(0.0, 1.0),
        Curve::Linear,
    )
}

pub(crate) const HAS_TARGET: Consideration = flag("has target", extract_has_target);
pub(crate) const NO_TARGET: Consideration = HAS_TARGET.inverted("no target");
pub(crate) const LINE_OF_SIGHT: Consideration = flag("line of sight", extract_line_of_sight);
pub(crate) const ANCHOR_PULL: Consideration = consideration(
    "anchor pull",
    extract_anchor_pull,
    Interval::new(0.0, 1.0),
    Curve::Cubic,
);
/// Empty above half health, full just past it: warriors break at ~50% hp.
pub(crate) const BADLY_HURT: Consideration = consideration(
    "badly hurt",
    extract_missing_health,
    Interval::new(0.45, 0.55),
    Curve::Linear,
);
/// A potion is worth drinking: steps from nothing to full just past 53%
/// missing health, because an action fires on any score above zero.
pub(crate) const NEEDS_POTION: Consideration = consideration(
    "needs potion",
    extract_missing_health,
    Interval::new(0.53, 0.535),
    Curve::Linear,
);
pub(crate) const NOT_DASHING: Consideration = flag(
    "dashing",
    extract_motion_is_running::<{ Motion::Dash.index() }>,
)
.inverted("not dashing");
pub(crate) const NOT_SWOOPING: Consideration = flag(
    "swooping",
    extract_motion_is_running::<{ Motion::Swoop.index() }>,
)
.inverted("not swooping");
pub(crate) const FAR_FROM_TARGET: Consideration =
    target_within_distance("near target", HEAL_SAFE_DISTANCE).inverted("far from target");
pub(crate) const RECOVERING: Consideration = flag("recovering mana", extract_recovering);
pub(crate) const NOT_RECOVERING: Consideration = RECOVERING.inverted("not recovering mana");
pub(crate) const ATTACK_FINISHED: Consideration = flag("attack finished", extract_attack_finished);
pub(crate) const CIRCLING: Consideration = flag("circling", extract_circling);

/// For the ice mage's escape dash, its reach is the danger ring.
pub(crate) const IN_DASH_REACH: Consideration = flag(
    "in dash reach",
    extract_in_reach::<{ motion_table_index(Motion::Dash) }>,
);
pub(crate) const IN_SWOOP_REACH: Consideration = flag(
    "in swoop reach",
    extract_in_reach::<{ motion_table_index(Motion::Swoop) }>,
);
pub(crate) const IN_MAINHAND_REACH: Consideration = flag(
    "in mainhand reach",
    extract_in_reach::<{ action_table_index(Action::Use(EquipmentSlot::Mainhand)) }>,
);
pub(crate) const IN_BREATH_REACH: Consideration = flag(
    "in breath reach",
    extract_in_reach::<{ action_table_index(Action::Ability(Ability::Breathe)) }>,
);
pub(crate) const IN_BITE_REACH: Consideration = flag(
    "in bite reach",
    extract_in_reach::<{ action_table_index(Action::Ability(Ability::Bite)) }>,
);
/// Zero inside contact reach, full a few units past it, so the bat only
/// breathes once biting stops being an option.
pub(crate) const BREATH_DISTANCE: Consideration = consideration(
    "breath distance",
    extract_target_distance,
    Interval::new(96.0, 100.0),
    Curve::Linear,
);

/// Distance over which cast quality ramps from zero at a distance-range edge to full.
const CAST_QUALITY_RAMP: f32 = 3.0;
const CAST_QUALITY_EDGE: f32 = CAST_QUALITY_RAMP / (CAST_MAX_RANGE - CAST_MIN_RANGE);

/// Shot quality over the cast distance range: full inside it, zero at both edges. The
/// ramps are short because an action fires on any score above zero.
pub(crate) const CAST_QUALITY: Consideration = consideration(
    "cast quality",
    extract_target_distance,
    CAST_DISTANCE_RANGE,
    Curve::Plateau {
        edge: CAST_QUALITY_EDGE,
    },
);

// Each pair below enters at one shared distance, so a body with no incumbent
// always has one of the two; the exits stay wide so an incumbent holds on.
const MELEE_ENTER: f32 = MELEE_RANGE + MELEE_HYSTERESIS / 2.0;

pub(crate) const OUTSIDE_MELEE_REACH: Consideration =
    distance_threshold("outside melee reach", MELEE_ENTER, MELEE_RANGE);
pub(crate) const INSIDE_MELEE_REACH: Consideration =
    distance_threshold("inside melee reach", MELEE_ENTER, MELEE_REACH);
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
pub(crate) const PAST_INNER_CAST_EDGE: Consideration = distance_threshold(
    "past inner cast edge",
    CAST_MIN_RANGE + CAST_HYSTERESIS,
    CAST_MIN_RANGE - CAST_HYSTERESIS,
);
