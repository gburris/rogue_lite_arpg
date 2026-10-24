//! Sense readers and scored facts for the brains. Readers report what they
//! know, always: `None` means the input itself is absent and the consideration
//! is skipped, never a "no".

use super::brains::MELEE_REACH;
use super::model::{
    Ability, CAST_MAX_RANGE, CAST_MIN_RANGE, Consideration, Curve, DecisionContext,
    HEAL_SAFE_DISTANCE, Maneuver, MotionMode, NormalizedRange,
};
use crate::items::prelude::EquipmentSlot;

fn flag_value(set: bool) -> Option<f32> {
    Some(if set { 1.0 } else { 0.0 })
}

fn read_has_target(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.target.is_some())
}

fn read_has_no_target(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.target.is_none())
}

fn read_line_of_sight(ctx: &DecisionContext) -> Option<f32> {
    ctx.target
        .map(|target| if target.line_of_sight { 1.0 } else { 0.0 })
}

fn read_in_melee_range(ctx: &DecisionContext) -> Option<f32> {
    // Match the hold gate's exit edge: a warrior parked by the arrival
    // deadband up to MELEE_REACH can still swing.
    ctx.target
        .map(|target| if target.distance <= MELEE_REACH { 1.0 } else { 0.0 })
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
    flag_value(ctx.anchor_arrived.is_some())
}

fn read_missing_health(ctx: &DecisionContext) -> Option<f32> {
    Some(1.0 - ctx.health_fraction)
}

fn read_mainhand_ready(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.slot_ready[EquipmentSlot::Mainhand as usize])
}

fn read_passive_idle(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.self_state.calm_phase == MotionMode::Idle)
}

fn read_passive_wander(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.self_state.calm_phase == MotionMode::Wander)
}

fn read_target_distance(ctx: &DecisionContext) -> Option<f32> {
    ctx.target.map(|target| target.distance)
}

/// Whether the maneuver at channel index `MANEUVER` is not the one running.
fn read_not_maneuver<const MANEUVER: usize>(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.self_state.active_maneuver.map(|maneuver| maneuver as usize) != Some(MANEUVER))
}

fn read_attack_finished(ctx: &DecisionContext) -> Option<f32> {
    flag_value(ctx.attack_finished)
}

fn read_circling(ctx: &DecisionContext) -> Option<f32> {
    flag_value(matches!(
        ctx.self_state.locomotion,
        MotionMode::Strafe | MotionMode::KeepDistance(_)
    ))
}

/// Whether the target is inside the trigger range of the maneuver at channel
/// index `MANEUVER`. A maneuver without a trigger range never reads as inside.
fn read_in_trigger_range<const MANEUVER: usize>(ctx: &DecisionContext) -> Option<f32> {
    let range = ctx.self_state.maneuver_tuning[MANEUVER].and_then(|tuning| tuning.trigger_range);
    let (Some(target), Some(range)) = (ctx.target, range) else {
        return Some(0.0);
    };
    flag_value(target.distance <= range)
}

/// Whether the target is inside the reach of the ability at channel index
/// `ABILITY`. An ability without a reach never reads as inside.
fn read_in_ability_reach<const ABILITY: usize>(ctx: &DecisionContext) -> Option<f32> {
    let reach = ctx.self_state.ability_tuning[ABILITY].and_then(|tuning| tuning.reach);
    let (Some(target), Some(reach)) = (ctx.target, reach) else {
        return Some(0.0);
    };
    flag_value(target.distance <= reach)
}

fn read_recovering(ctx: &DecisionContext) -> Option<f32> {
    // A mana break is only meaningful while the pool is still short; the
    // held state supplies the hysteresis and the fraction keeps the sense
    // honest if a caster ever starts a break from a full pool.
    flag_value(ctx.self_state.mana_recovering && ctx.mana_fraction < 1.0)
}

fn read_not_recovering(ctx: &DecisionContext) -> Option<f32> {
    flag_value(!ctx.self_state.mana_recovering)
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
pub(crate) const NOT_SWOOPING: Consideration = flag(
    "not swooping",
    read_not_maneuver::<{ Maneuver::Swoop as usize }>,
);
pub(crate) const WITHIN_CAST_RANGE: Consideration =
    flag("within cast range", read_within_cast_range);
pub(crate) const FAR_FROM_TARGET: Consideration = flag("far from target", read_far_from_target);
pub(crate) const NOT_KITING: Consideration = flag(
    "not kiting",
    read_not_maneuver::<{ Maneuver::Kite as usize }>,
);
pub(crate) const RECOVERING: Consideration = flag("recovering mana", read_recovering);
pub(crate) const NOT_RECOVERING: Consideration = flag("not recovering mana", read_not_recovering);
pub(crate) const ATTACK_FINISHED: Consideration = flag("attack finished", read_attack_finished);
pub(crate) const CIRCLING: Consideration = flag("circling", read_circling);
pub(crate) const IN_DANGER_RING: Consideration = flag(
    "in danger ring",
    read_in_trigger_range::<{ Maneuver::Slide as usize }>,
);
pub(crate) const IN_BITE_REACH: Consideration = flag(
    "in bite reach",
    read_in_ability_reach::<{ Ability::Bite as usize }>,
);

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
