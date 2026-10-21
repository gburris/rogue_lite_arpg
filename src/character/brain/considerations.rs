//! Sense readers and scored facts for the brains.
//!
//! Each reader pulls one plain value out of a [`DecisionContext`]. [`flag`]
//! turns a yes/no fact into a consideration, and [`ramp_above`] turns a
//! growing measurement (like anchor-range overshoot) into one that fills up
//! over a one-unit window.
//!
//! Reader contract: readers report what they know, always. A fact the entity
//! can observe returns `Some(1.0)` or `Some(0.0)` — never `None` as a "no".
//! `None` means the input itself is absent (no target, no anchor) and the
//! consideration is skipped without affecting the score, so absence can never
//! silently veto an option. Options that need an absent input carry an
//! explicit presence fact instead (`HAS_TARGET`, `HAS_ANCHOR`).

use super::model::{
    Consideration, Curve, DecisionContext, HEAL_SAFE_DISTANCE, MAGE_MAX_RANGE, MAGE_MIN_RANGE,
    MELEE_RANGE, MotionMode,
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
        if target.distance <= MELEE_RANGE {
            1.0
        } else {
            0.0
        }
    })
}

fn read_outside_melee_range(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        if target.distance > MELEE_RANGE {
            1.0
        } else {
            0.0
        }
    })
}

fn read_within_cast_range(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        if target.distance <= MAGE_MAX_RANGE {
            1.0
        } else {
            0.0
        }
    })
}

fn read_beyond_cast_range(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        if target.distance > MAGE_MAX_RANGE {
            1.0
        } else {
            0.0
        }
    })
}
fn read_beyond_min_cast_range(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| {
        if target.distance >= MAGE_MIN_RANGE {
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

const fn consideration(
    name: &'static str,
    read: fn(&DecisionContext) -> Option<f32>,
    range: (f32, f32),
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
    consideration(name, read, (0.0, 1.0), Curve::Linear)
}

const fn ramp_above(
    name: &'static str,
    read: fn(&DecisionContext) -> Option<f32>,
    at: f32,
) -> Consideration {
    consideration(name, read, (at, at + 1.0), Curve::Linear)
}

pub(crate) const HAS_TARGET: Consideration = flag("has target", read_has_target);
pub(crate) const NO_TARGET: Consideration = flag("no target", read_has_no_target);
pub(crate) const LINE_OF_SIGHT: Consideration = flag("line of sight", read_line_of_sight);
pub(crate) const MAINHAND_READY: Consideration = flag("mainhand ready", read_mainhand_ready);
pub(crate) const PASSIVE_IDLE: Consideration = flag("passive idle", read_passive_idle);
pub(crate) const PASSIVE_WANDER: Consideration = flag("passive wander", read_passive_wander);
pub(crate) const BEYOND_ANCHOR_RANGE: Consideration =
    ramp_above("beyond anchor range", read_anchor_overshoot_fraction, 0.0);
pub(crate) const HAS_ANCHOR: Consideration = flag("has anchor", read_has_anchor);
/// Empty above half health, full just past it: warriors break at ~50% hp.
pub(crate) const BADLY_HURT: Consideration = consideration(
    "badly hurt",
    read_missing_health,
    (0.45, 0.55),
    Curve::Linear,
);
pub(crate) const IN_MELEE_RANGE: Consideration = flag("in melee range", read_in_melee_range);
pub(crate) const OUTSIDE_MELEE_RANGE: Consideration =
    flag("outside melee range", read_outside_melee_range);
pub(crate) const WITHIN_CAST_RANGE: Consideration =
    flag("within cast range", read_within_cast_range);
pub(crate) const BEYOND_CAST_RANGE: Consideration =
    flag("beyond cast range", read_beyond_cast_range);
pub(crate) const BEYOND_MIN_CAST_RANGE: Consideration =
    flag("beyond min cast range", read_beyond_min_cast_range);
/// Far enough from the target to drink safely. Gates warrior healing so the
/// potion only fires after the flee has opened real distance.
pub(crate) const FAR_FROM_TARGET: Consideration =
    flag("far from target", read_far_from_target);
