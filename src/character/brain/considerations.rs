//! Sense readers and scored facts for the brains. Readers report what they
//! know, always: `None` means the input itself is absent and the consideration
//! is skipped, never a "no".

use super::brains::MELEE_HYSTERESIS;
use super::model::{
    CAST_MAX_RANGE, CAST_MIN_RANGE, Consideration, Curve, DecisionContext, HEAL_SAFE_DISTANCE,
    MELEE_RANGE, MotionMode, NormalizedRange,
};
use crate::items::prelude::EquipmentSlot;
fn read_has_target(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.target.is_some() { 1.0 } else { 0.0 })
}

fn read_has_no_target(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.target.is_none() { 1.0 } else { 0.0 })
}

fn read_line_of_sight(ctx: &DecisionContext) -> Option<f32> {
    ctx.target
        .map(|target| if target.line_of_sight { 1.0 } else { 0.0 })
}

fn read_in_melee_range(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        // Match the hold gate's exit edge: a warrior parked by the arrival
        // deadband up to MELEE_RANGE + hysteresis can still swing.
        if target.distance <= MELEE_RANGE + MELEE_HYSTERESIS {
            1.0
        } else {
            0.0
        }
    })
}

fn read_within_cast_range(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        if target.distance <= CAST_MAX_RANGE {
            1.0
        } else {
            0.0
        }
    })
}

fn read_far_from_target(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        if target.distance >= HEAL_SAFE_DISTANCE {
            1.0
        } else {
            0.0
        }
    })
}

const fn read_anchor_overshoot_fraction(ctx: &DecisionContext) -> Option<f32> {
    ctx.anchor_overshoot_fraction
}

fn read_has_anchor(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.anchor_arrived.is_some() {
        1.0
    } else {
        0.0
    })
}

fn read_missing_health(ctx: &DecisionContext) -> Option<f32> {
    Some(1.0 - ctx.health_fraction)
}

fn read_mainhand_ready(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.slot_ready[EquipmentSlot::Mainhand as usize] {
        1.0
    } else {
        0.0
    })
}

fn read_passive_idle(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.calm_phase == MotionMode::Idle {
        1.0
    } else {
        0.0
    })
}

fn read_passive_wander(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.calm_phase == MotionMode::Wander {
        1.0
    } else {
        0.0
    })
}
fn read_swoop_called(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.swoop_called { 1.0 } else { 0.0 })
}
fn read_not_swooping(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.swoop_called { 0.0 } else { 1.0 })
}

fn read_target_distance(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| target.distance)
}

fn read_kite_called(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.kite_called { 1.0 } else { 0.0 })
}

fn read_not_kiting(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.kite_called { 0.0 } else { 1.0 })
}

fn read_burst_called(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.burst_called { 1.0 } else { 0.0 })
}

fn read_slide_called(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.slide_called { 1.0 } else { 0.0 })
}
fn read_recovering(ctx: &DecisionContext) -> Option<f32> {
    // A mana break is only meaningful while the pool is still short; the
    // latched state supplies the hysteresis and the fraction keeps the sense
    // honest if a caster ever starts a break from a full pool.
    Some(if ctx.mana_recovering && ctx.mana_fraction < 1.0 {
        1.0
    } else {
        0.0
    })
}

fn read_not_recovering(ctx: &DecisionContext) -> Option<f32> {
    Some(if ctx.mana_recovering { 0.0 } else { 1.0 })
}

const fn consideration(
    name: &'static str,
    read: fn(&DecisionContext) -> Option<f32>,
    range: NormalizedRange,
    curve: Curve,
) -> Consideration {
    Consideration {
        name,
        read,
        range,
        curve,
    }
}

const fn flag(name: &'static str, read: fn(&DecisionContext) -> Option<f32>) -> Consideration {
    consideration(name, read, NormalizedRange::new(0.0, 1.0), Curve::Linear)
}

pub(crate) const HAS_TARGET: Consideration = flag("has target", read_has_target);
pub(crate) const NO_TARGET: Consideration = flag("no target", read_has_no_target);
pub(crate) const LINE_OF_SIGHT: Consideration = flag("line of sight", read_line_of_sight);
pub(crate) const MAINHAND_READY: Consideration = flag("mainhand ready", read_mainhand_ready);
pub(crate) const PASSIVE_IDLE: Consideration = flag("passive idle", read_passive_idle);
pub(crate) const PASSIVE_WANDER: Consideration = flag("passive wander", read_passive_wander);
pub(crate) const BEYOND_ANCHOR_RANGE: Consideration = consideration(
    "beyond anchor range",
    read_anchor_overshoot_fraction,
    NormalizedRange::new(0.0, 1.0),
    Curve::Cubic,
);
pub(crate) const HAS_ANCHOR: Consideration = flag("has anchor", read_has_anchor);
/// Empty above half health, full just past it: warriors break at ~50% hp.
pub(crate) const BADLY_HURT: Consideration = consideration(
    "badly hurt",
    read_missing_health,
    NormalizedRange::new(0.45, 0.55),
    Curve::Linear,
);
pub(crate) const IN_MELEE_RANGE: Consideration = flag("in melee range", read_in_melee_range);
pub(crate) const SWOOP_CALLED: Consideration = flag("swoop called", read_swoop_called);
pub(crate) const NOT_SWOOPING: Consideration = flag("not swooping", read_not_swooping);
pub(crate) const WITHIN_CAST_RANGE: Consideration =
    flag("within cast range", read_within_cast_range);
pub(crate) const FAR_FROM_TARGET: Consideration = flag("far from target", read_far_from_target);
pub(crate) const KITE_CALLED: Consideration = flag("kite called", read_kite_called);
pub(crate) const NOT_KITING: Consideration = flag("not kiting", read_not_kiting);
pub(crate) const BURST_CALLED: Consideration = flag("burst called", read_burst_called);
pub(crate) const SLIDE_CALLED: Consideration = flag("slide called", read_slide_called);
pub(crate) const RECOVERING: Consideration = flag("recovering mana", read_recovering);
pub(crate) const NOT_RECOVERING: Consideration = flag("not recovering mana", read_not_recovering);

const CAST_QUALITY_MARGIN: f32 = 20.0;
const CAST_QUALITY_EDGE: f32 =
    2.0 * CAST_QUALITY_MARGIN / (CAST_MAX_RANGE - CAST_MIN_RANGE + 2.0 * CAST_QUALITY_MARGIN);
const CAST_QUALITY_RANGE: NormalizedRange = NormalizedRange::new(
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
