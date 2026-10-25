//! Sense readers and scored facts for the brains. Readers report what they
//! know, always: `None` means the input itself is absent (no target, no
//! anchor), which vetoes the decision. A consideration that reads the target
//! therefore implies one.

use super::model::{
    Ability, Action, CAST_MAX_RANGE, CAST_MIN_RANGE, Consideration, Curve, DecisionContext,
    HEAL_SAFE_DISTANCE, Interval, MELEE_RANGE, Maneuver, MotionMode, Response, Tuning,
};
use crate::items::prelude::EquipmentSlot;

pub(crate) const MELEE_HYSTERESIS: f32 = 4.0;
/// Melee reach including the hold threshold's hysteresis: where a parked warrior can still swing.
pub(crate) const MELEE_REACH: f32 = MELEE_RANGE + MELEE_HYSTERESIS;
const CAST_HYSTERESIS: f32 = 5.0;

fn flag_value(set: bool) -> Option<f32> {
    Some(if set { 1.0 } else { 0.0 })
}

fn read_has_target(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.target.is_some())
}

fn read_line_of_sight(ctx: &DecisionContext) -> Option<f32> {
    ctx.target
        .map(|target| if target.line_of_sight { 1.0 } else { 0.0 })
}

/// How far past its anchor range the body is. Once already heading home it
/// reads full until arrival, so crossing back inside the range mid-return
/// cannot hand the body straight back to calm.
fn read_anchor_pull(ctx: &DecisionContext) -> Option<f32> {
    if ctx.mode == MotionMode::ReturnToAnchor {
        ctx.anchor_arrived
            .map(|arrived| if arrived { 0.0 } else { 1.0 })
    } else {
        ctx.anchor_overshoot_fraction
    }
}

fn read_missing_health(ctx: &DecisionContext) -> Option<f32> {
    Some(1.0 - ctx.health_fraction)
}

fn read_mainhand_ready(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.slot_ready[EquipmentSlot::Mainhand as usize])
}

fn read_target_distance(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| target.distance)
}

/// Whether the maneuver at index `MANEUVER` is the one running.
fn read_running<const MANEUVER: usize>(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.active_maneuver.map(|maneuver| maneuver as usize) == Some(MANEUVER))
}

fn read_attack_finished(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.attack_finished)
}

fn read_circling(ctx: &DecisionContext) -> Option<f32> {
    flag_value(matches!(
        ctx.mode,
        MotionMode::Strafe(_) | MotionMode::KeepDistance(_)
    ))
}

/// Whether the target is inside a capability's reach. A capability without a
/// reach, or no target at all, never reads as inside.
fn in_reach(ctx: &DecisionContext, tuning: Option<Tuning>) -> Option<f32> {
    let reach = tuning.and_then(|tuning| tuning.reach);
    let (Some(target), Some(reach)) = (ctx.target, reach) else {
        return Some(0.0);
    };
    flag_value(target.distance <= reach)
}

fn read_maneuver_in_reach<const MANEUVER: usize>(ctx: &DecisionContext) -> Option<f32> {
    in_reach(ctx, ctx.maneuver_tuning[MANEUVER])
}

fn read_action_in_reach<const ACTION: usize>(ctx: &DecisionContext) -> Option<f32> {
    in_reach(ctx, ctx.action_tuning[ACTION])
}

fn read_recovering(ctx: &DecisionContext) -> Option<f32> {
    // A mana break is only meaningful while the pool is still short; the
    // held state supplies the hysteresis and the fraction keeps the sense
    // honest if a caster ever starts a break from a full pool.
    flag_value(ctx.mana_recovering && ctx.mana_fraction < 1.0)
}

const fn consideration(
    name: &'static str,
    read: fn(&DecisionContext) -> Option<f32>,
    range: Interval,
    curve: Curve,
) -> Consideration {
    Consideration {
        name,
        read,
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
        read: read_target_distance,
        response: Response::Threshold { enter, exit },
        inverted: false,
    }
}

/// Target at most `distance` away, with no hysteresis.
const fn within(name: &'static str, distance: f32) -> Consideration {
    distance_threshold(name, distance, distance)
}

const fn flag(name: &'static str, read: fn(&DecisionContext) -> Option<f32>) -> Consideration {
    consideration(name, read, Interval::new(0.0, 1.0), Curve::Linear)
}

pub(crate) const HAS_TARGET: Consideration = flag("has target", read_has_target);
pub(crate) const NO_TARGET: Consideration = HAS_TARGET.inverted("no target");
pub(crate) const LINE_OF_SIGHT: Consideration = flag("line of sight", read_line_of_sight);
pub(crate) const MAINHAND_READY: Consideration = flag("mainhand ready", read_mainhand_ready);
pub(crate) const ANCHOR_PULL: Consideration = consideration(
    "anchor pull",
    read_anchor_pull,
    Interval::new(0.0, 1.0),
    Curve::Cubic,
);
/// Empty above half health, full just past it: warriors break at ~50% hp.
pub(crate) const BADLY_HURT: Consideration = consideration(
    "badly hurt",
    read_missing_health,
    Interval::new(0.45, 0.55),
    Curve::Linear,
);
pub(crate) const NOT_DASHING: Consideration =
    flag("dashing", read_running::<{ Maneuver::Dash as usize }>).inverted("not dashing");
pub(crate) const NOT_SWOOPING: Consideration =
    flag("swooping", read_running::<{ Maneuver::Swoop as usize }>).inverted("not swooping");
pub(crate) const FAR_FROM_TARGET: Consideration =
    within("near target", HEAL_SAFE_DISTANCE).inverted("far from target");
pub(crate) const RECOVERING: Consideration = flag("recovering mana", read_recovering);
pub(crate) const NOT_RECOVERING: Consideration = RECOVERING.inverted("not recovering mana");
pub(crate) const ATTACK_FINISHED: Consideration = flag("attack finished", read_attack_finished);
pub(crate) const CIRCLING: Consideration = flag("circling", read_circling);

/// For the ice mage's escape dash, its reach is the danger ring.
pub(crate) const IN_DASH_REACH: Consideration = flag(
    "in dash reach",
    read_maneuver_in_reach::<{ Maneuver::Dash as usize }>,
);
pub(crate) const IN_SWOOP_REACH: Consideration = flag(
    "in swoop reach",
    read_maneuver_in_reach::<{ Maneuver::Swoop as usize }>,
);
pub(crate) const IN_MAINHAND_REACH: Consideration = flag(
    "in mainhand reach",
    read_action_in_reach::<{ Action::Use(EquipmentSlot::Mainhand).index() }>,
);
pub(crate) const IN_BREATH_REACH: Consideration = flag(
    "in breath reach",
    read_action_in_reach::<{ Action::Ability(Ability::Breathe).index() }>,
);
pub(crate) const IN_BITE_REACH: Consideration = flag(
    "in bite reach",
    read_action_in_reach::<{ Action::Ability(Ability::Bite).index() }>,
);
/// Breath appeal rising with distance, from zero where a bite lands to full a
/// short way past it, so the bat only breathes once biting stops being an option.
pub(crate) const BREATH_DISTANCE: Consideration = consideration(
    "breath distance",
    read_target_distance,
    Interval::new(70.0, 110.0),
    Curve::Linear,
);

const CAST_QUALITY_MARGIN: f32 = 20.0;
const CAST_QUALITY_EDGE: f32 =
    2.0 * CAST_QUALITY_MARGIN / (CAST_MAX_RANGE - CAST_MIN_RANGE + 2.0 * CAST_QUALITY_MARGIN);
const CAST_QUALITY_RANGE: Interval = Interval::new(
    CAST_MIN_RANGE - CAST_QUALITY_MARGIN,
    CAST_MAX_RANGE + CAST_QUALITY_MARGIN,
);

/// Shot quality over the cast band: full through the core, easing to zero at both edges.
pub(crate) const CAST_QUALITY: Consideration = consideration(
    "cast quality",
    read_target_distance,
    CAST_QUALITY_RANGE,
    Curve::Plateau {
        edge: CAST_QUALITY_EDGE,
    },
);

pub(crate) const OUTSIDE_MELEE_REACH: Consideration =
    distance_threshold("outside melee reach", MELEE_REACH, MELEE_RANGE);
pub(crate) const INSIDE_MELEE_REACH: Consideration =
    distance_threshold("inside melee reach", MELEE_RANGE, MELEE_REACH);
pub(crate) const BEYOND_CAST_BAND: Consideration = distance_threshold(
    "beyond cast band",
    CAST_MAX_RANGE + CAST_HYSTERESIS,
    CAST_MAX_RANGE - CAST_HYSTERESIS,
);
pub(crate) const INSIDE_OUTER_CAST_EDGE: Consideration = distance_threshold(
    "inside outer cast edge",
    CAST_MAX_RANGE - CAST_HYSTERESIS,
    CAST_MAX_RANGE + CAST_HYSTERESIS,
);
pub(crate) const PAST_INNER_CAST_EDGE: Consideration = distance_threshold(
    "past inner cast edge",
    CAST_MIN_RANGE + CAST_HYSTERESIS,
    CAST_MIN_RANGE - CAST_HYSTERESIS,
);
