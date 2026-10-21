//! Scoring math for the utility brains.
//!
//! [`select_best`] picks the winning option, [`score_option`] prices one
//! option, and [`decide`] produces a full [`BrainDecision`] for a tick.
//! [`validate_brain`] rejects malformed definitions at startup, before they
//! can silently produce nonsense.
//!
//! Arbitration is two-level, reasoner-style: [`decide`] first scores each
//! [`BehaviorGroup`] on pure merit and only then picks the best option inside
//! the winning group. Groups never compete on tuned priorities, so calm
//! stickiness cannot suppress a reactive move.

use super::model::{
    BehaviorGroup, Brain, BrainDecision, Consideration, DecisionContext, MotionMode, ScoredOption,
};
use crate::items::prelude::EquipmentSlot;

/// Flat score bonus for staying in the current mode while its commitment lasts.
///
/// The bonus only breaks ties *within* the already-winning group: groups
/// are arbitrated on bonus-free merit first, so a committed calm mode can
/// never outbid a reactive move. For shipped brains the passive group never
/// fields two live options at once (the calm phase admits exactly one), which
/// keeps idle/wander alternation crisp without dithering.
const COMMITMENT_BONUS: f32 = 2.0;

/// Selects the best option index. Options scoring zero or lower are not
/// selectable, and ties go to the first definition because comparison is strict
/// (`>`).
pub fn select_best<T>(
    options: &[ScoredOption<T>],
    ctx: &DecisionContext,
    current: Option<usize>,
) -> Option<usize> {
    select_best_where(options, ctx, current, |_| true)
}

fn select_best_where<T>(
    options: &[ScoredOption<T>],
    ctx: &DecisionContext,
    current: Option<usize>,
    mut include: impl FnMut(&T) -> bool,
) -> Option<usize> {
    let mut best = None;

    for (index, option) in options.iter().enumerate() {
        if !include(&option.value) {
            continue;
        }
        let Some(score) = score_option(option, ctx, current == Some(index)) else {
            continue;
        };
        if score <= 0.0 {
            continue;
        }

        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((index, score)),
        }
    }

    best.map(|(index, _)| index)
}

/// Final bounded score for one option, including commitment stickiness.
pub fn score_option<T>(
    option: &ScoredOption<T>,
    ctx: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    if !option.priority.is_finite() || option.priority < 0.0 {
        return None;
    }

    let mut score = option.priority;
    for consideration in option.considerations {
        score *= score_consideration(consideration, ctx)?;
    }

    if is_current && ctx.within_commitment && score > 0.0 {
        score += COMMITMENT_BONUS;
    }

    (score.is_finite() && score >= 0.0).then_some(score)
}

/// Whether the brain wants to use the item in `slot` this tick.
pub fn choose_slot(brain: &Brain, slot: EquipmentSlot, ctx: &DecisionContext) -> bool {
    select_best(brain.slot_options[slot as usize], ctx, None).is_some()
}

/// Produces the complete movement and slot-intent decision for one tick.
pub fn decide(brain: &Brain, ctx: &DecisionContext, current: MotionMode) -> BrainDecision {
    let mut slot_intents = [false; EquipmentSlot::ALL.len()];
    for slot in EquipmentSlot::ALL {
        slot_intents[slot as usize] = choose_slot(brain, slot, ctx);
    }
    let wants_heal = select_best(brain.heal_options, ctx, None).is_some();

    let current_index = brain
        .motion_options
        .iter()
        .position(|option| option.value == current);

    // Once a character starts returning to its anchor, keep the mode latched
    // until it actually arrives. Re-scoring the anchor condition alone drops
    // the return score to zero the moment it crosses the anchor-range
    // boundary, which lets wander immediately pull it back out and produces
    // boundary zig-zagging.
    let keep_returning = current == MotionMode::ReturnToAnchor
        && ctx.target.is_none()
        && ctx.anchor_arrived == Some(false);

    let selected = if keep_returning {
        current_index.and_then(|index| brain.motion_options.get(index))
    } else {
        let group = pick_group(brain.motion_options, ctx);
        select_best_where(brain.motion_options, ctx, current_index, |mode| {
            mode.group() == group
        })
        .and_then(|index| brain.motion_options.get(index))
    };

    BrainDecision {
        motion: selected.map_or(current, |option| option.value),
        commit: selected.and_then(|option| option.commit),
        slot_intents,
        wants_heal,
    }
}

/// Picks the winning behavior group on bonus-free merit. Structural order
/// breaks ties — a live target always means [`BehaviorGroup::Aggro`] — and an
/// empty field defaults to [`BehaviorGroup::Passive`], which hands control back to
/// the current mode downstream when nothing scores.
fn pick_group(options: &[ScoredOption<MotionMode>], ctx: &DecisionContext) -> BehaviorGroup {
    let mut scores = [
        (BehaviorGroup::Aggro, 0.0f32),
        (BehaviorGroup::Return, 0.0f32),
        (BehaviorGroup::Passive, 0.0f32),
    ];
    for option in options {
        if let Some(entry) = scores
            .iter_mut()
            .find(|(group, _)| *group == option.value.group())
            && let Some(score) = score_option(option, ctx, false)
        {
            entry.1 = entry.1.max(score);
        }
    }

    let mut winner = BehaviorGroup::Passive;
    let mut winner_score = 0.0;
    for (group, score) in scores {
        if score > winner_score {
            winner = group;
            winner_score = score;
        }
    }
    winner
}

/// Rejects malformed brains before they can silently produce nonsense.
pub fn validate_brain(brain: &Brain) -> Result<(), String> {
    if brain.motion_options.is_empty() {
        return Err("motion list is empty".to_string());
    }

    for (index, option) in brain.motion_options.iter().enumerate() {
        validate_option(option).map_err(|error| format!("motion option {index}: {error}"))?;
        validate_motion(option.value).map_err(|error| format!("motion option {index}: {error}"))?;

        if brain.motion_options[..index]
            .iter()
            .any(|earlier| earlier.value == option.value)
        {
            return Err(format!("motion option {index} duplicates an earlier mode"));
        }
    }

    for (slot_index, options) in brain.slot_options.iter().enumerate() {
        for (option_index, option) in options.iter().enumerate() {
            validate_option(option)
                .map_err(|error| format!("slot {slot_index} option {option_index}: {error}"))?;
        }
    }
    for (option_index, option) in brain.heal_options.iter().enumerate() {
        validate_option(option)
            .map_err(|error| format!("heal option {option_index}: {error}"))?;
    }

    Ok(())
}

fn validate_option<T>(option: &ScoredOption<T>) -> Result<(), String> {
    if !option.priority.is_finite() || option.priority <= 0.0 {
        return Err("priority must be finite and positive".to_string());
    }

    if let Some(window) = option.commit
        && (!window.min.is_finite()
            || !window.max.is_finite()
            || window.min < 0.0
            || window.max < 0.0
            || window.min > window.max)
    {
        return Err("commit bounds must be finite, non-negative, and ordered".to_string());
    }

    for consideration in option.considerations {
        let (min, max) = consideration.range;
        if !min.is_finite() || !max.is_finite() || min >= max {
            return Err(format!(
                "consideration '{}' has an invalid range",
                consideration.name
            ));
        }
    }

    Ok(())
}

fn validate_motion(motion: MotionMode) -> Result<(), String> {
    if let MotionMode::KeepDistance { min, max } = motion
        && (!min.is_finite() || !max.is_finite() || min >= max)
    {
        return Err("KeepDistance requires finite min < max".to_string());
    }
    Ok(())
}

enum ConsiderationValue {
    Unavailable,
    Value(f32),
    Invalid,
}

fn evaluate_consideration(
    consideration: &Consideration,
    ctx: &DecisionContext,
) -> ConsiderationValue {
    let Some(raw) = (consideration.read)(ctx) else {
        return ConsiderationValue::Unavailable;
    };
    normalize_raw(consideration, raw).map_or(ConsiderationValue::Invalid, ConsiderationValue::Value)
}

fn score_consideration(consideration: &Consideration, ctx: &DecisionContext) -> Option<f32> {
    match evaluate_consideration(consideration, ctx) {
        // A missing input is neutral: readers only return `None` for inputs
        // that are absent (no target, no anchor), never for a "no".
        ConsiderationValue::Unavailable => Some(1.0),
        ConsiderationValue::Value(value) => Some(value),
        ConsiderationValue::Invalid => None,
    }
}

fn normalize_raw(consideration: &Consideration, raw: f32) -> Option<f32> {
    if !raw.is_finite() {
        return None;
    }

    let (min, max) = consideration.range;
    if !min.is_finite() || !max.is_finite() || min >= max {
        return None;
    }

    let normalized = ((raw - min) / (max - min)).clamp(0.0, 1.0);
    if !normalized.is_finite() {
        return None;
    }

    consideration.curve.apply(normalized)
}

#[cfg(test)]
mod tests {
    use super::super::brains::{RANGED_MAGE, VILLAGER, WARRIOR};
    use super::super::model::{HEAL_SAFE_DISTANCE, MAGE_MAX_RANGE, MAGE_MIN_RANGE, MELEE_RANGE, TargetView};
    use super::*;

    fn context() -> DecisionContext {
        DecisionContext::default()
    }

    fn mainhand_ready() -> [bool; EquipmentSlot::ALL.len()] {
        let mut slot_ready = [false; EquipmentSlot::ALL.len()];
        slot_ready[EquipmentSlot::Mainhand as usize] = true;
        slot_ready
    }

    fn visible_target(distance: f32) -> TargetView {
        TargetView {
            distance,
            line_of_sight: true,
        }
    }

    fn warrior_context() -> DecisionContext {
        DecisionContext {
            target: Some(visible_target(30.0)),
            slot_ready: mainhand_ready(),
            ..context()
        }
    }

    #[test]
    fn no_target_uses_the_next_calm_phase() {
        let ctx = DecisionContext {
            calm_phase: MotionMode::Wander,
            ..context()
        };
        let decision = decide(&VILLAGER, &ctx, MotionMode::Idle);
        assert_eq!(decision.motion, MotionMode::Wander);
    }

    #[test]
    fn warrior_in_reach_strafes_and_requests_mainhand() {
        let ctx = warrior_context();
        let decision = decide(&WARRIOR, &ctx, MotionMode::Chase);

        assert_eq!(decision.motion, MotionMode::Strafe);
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn warrior_out_of_reach_chases_without_requesting_mainhand() {
        let ctx = DecisionContext {
            target: Some(visible_target(120.0)),
            ..warrior_context()
        };
        let decision = decide(&WARRIOR, &ctx, MotionMode::Idle);

        assert_eq!(decision.motion, MotionMode::Chase);
        assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn cooldown_blocks_slot_request_without_changing_motion() {
        let ctx = DecisionContext {
            slot_ready: [false; EquipmentSlot::ALL.len()],
            ..warrior_context()
        };
        let decision = decide(&WARRIOR, &ctx, MotionMode::Idle);

        assert_eq!(
            decision.motion,
            MotionMode::KeepDistance {
                min: 0.0,
                max: MELEE_RANGE
            }
        );
        assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn warrior_flees_from_target_when_badly_hurt() {
        let fleeing = DecisionContext {
            target: Some(visible_target(30.0)),
            slot_ready: mainhand_ready(),
            health_fraction: 0.1,
            anchor_overshoot_fraction: Some(0.0),
            anchor_arrived: Some(false),
            ..context()
        };
        let decision = decide(&WARRIOR, &fleeing, MotionMode::Chase);
        assert_eq!(decision.motion, MotionMode::Flee);
        // Still swinging on the way out: a fighting retreat, not a rout.
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
        let scratched = DecisionContext {
            health_fraction: 0.5,
            ..fleeing
        };
        let decision = decide(&WARRIOR, &scratched, MotionMode::Chase);
        assert_eq!(decision.motion, MotionMode::Strafe);
    }

    #[test]
    fn mage_strafes_while_ready_and_fires() {
        let ctx = DecisionContext {
            target: Some(visible_target(250.0)),
            slot_ready: mainhand_ready(),
            ..context()
        };
        let decision = decide(&RANGED_MAGE, &ctx, MotionMode::Idle);

        assert_eq!(decision.motion, MotionMode::Strafe);
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn mage_holds_range_while_cooling_down() {
        let ctx = DecisionContext {
            target: Some(visible_target(250.0)),
            slot_ready: [false; EquipmentSlot::ALL.len()],
            ..context()
        };
        let decision = decide(&RANGED_MAGE, &ctx, MotionMode::Strafe);

        assert_eq!(
            decision.motion,
            MotionMode::KeepDistance {
                min: MAGE_MIN_RANGE,
                max: MAGE_MAX_RANGE
            }
        );
        assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn mage_chases_beyond_its_band() {
        let ctx = DecisionContext {
            target: Some(visible_target(MAGE_MAX_RANGE + 50.0)),
            slot_ready: mainhand_ready(),
            ..context()
        };
        let decision = decide(
            &RANGED_MAGE,
            &ctx,
            MotionMode::KeepDistance {
                min: MAGE_MIN_RANGE,
                max: MAGE_MAX_RANGE,
            },
        );

        assert_eq!(decision.motion, MotionMode::Chase);
    }

    #[test]
    fn mage_without_line_of_sight_does_not_request_mainhand() {
        let ctx = DecisionContext {
            target: Some(TargetView {
                distance: 250.0,
                line_of_sight: false,
            }),
            slot_ready: mainhand_ready(),
            ..context()
        };
        let decision = decide(&RANGED_MAGE, &ctx, MotionMode::Idle);

        assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }
    #[test]
    fn villager_calm_phase_and_anchor_choose_expected_modes() {
        let wander_phase = DecisionContext {
            calm_phase: MotionMode::Wander,
            anchor_overshoot_fraction: Some(0.0),
            anchor_arrived: Some(false),
            ..context()
        };
        assert_eq!(
            decide(&VILLAGER, &wander_phase, MotionMode::Idle).motion,
            MotionMode::Wander
        );

        let idle_phase = DecisionContext {
            calm_phase: MotionMode::Idle,
            ..wander_phase
        };
        assert_eq!(
            decide(&VILLAGER, &idle_phase, MotionMode::Wander).motion,
            MotionMode::Idle
        );

        // Just past the boundary the deadband holds: no twitchy about-face.
        let near = DecisionContext {
            anchor_overshoot_fraction: Some(0.1),
            ..wander_phase
        };
        assert_eq!(
            decide(&VILLAGER, &near, MotionMode::Idle).motion,
            MotionMode::Wander
        );

        let far = DecisionContext {
            anchor_overshoot_fraction: Some(1.5),
            ..wander_phase
        };
        assert_eq!(
            decide(&VILLAGER, &far, MotionMode::Wander).motion,
            MotionMode::ReturnToAnchor
        );
    }

    #[test]
    fn return_latch_holds_until_anchor_arrival() {
        let returning = DecisionContext {
            anchor_overshoot_fraction: Some(0.0),
            anchor_arrived: Some(false),
            ..context()
        };
        assert_eq!(
            decide(&VILLAGER, &returning, MotionMode::ReturnToAnchor).motion,
            MotionMode::ReturnToAnchor
        );

        let arrived = DecisionContext {
            anchor_arrived: Some(true),
            ..returning
        };
        assert_eq!(
            decide(&VILLAGER, &arrived, MotionMode::ReturnToAnchor).motion,
            MotionMode::Idle
        );

        let contact = DecisionContext {
            target: Some(visible_target(120.0)),
            slot_ready: mainhand_ready(),
            anchor_arrived: Some(false),
            ..context()
        };
        assert_eq!(
            decide(&WARRIOR, &contact, MotionMode::ReturnToAnchor).motion,
            MotionMode::Chase
        );
    }

    #[test]
    fn committed_calm_does_not_delay_return() {
        let ctx = DecisionContext {
            calm_phase: MotionMode::Wander,
            anchor_overshoot_fraction: Some(0.3),
            anchor_arrived: Some(false),
            within_commitment: true,
            ..context()
        };
        assert_eq!(
            decide(&VILLAGER, &ctx, MotionMode::Wander).motion,
            MotionMode::ReturnToAnchor
        );
    }

    #[test]
    fn commitment_breaks_a_marginal_score_difference() {
        static STICKY_MOTION: &[ScoredOption<MotionMode>] = &[
            ScoredOption {
                value: MotionMode::Idle,
                priority: 0.5,
                commit: None,
                considerations: &[],
            },
            ScoredOption {
                value: MotionMode::Wander,
                priority: 0.6,
                commit: None,
                considerations: &[],
            },
        ];
        static STICKY: Brain = Brain {
            motion_options: STICKY_MOTION,
            slot_options: super::super::brains::NO_SLOTS,
            heal_options: &[],
        };

        let committed = DecisionContext {
            within_commitment: true,
            ..context()
        };
        assert_eq!(
            decide(&STICKY, &committed, MotionMode::Idle).motion,
            MotionMode::Idle
        );
        assert_eq!(
            decide(&STICKY, &context(), MotionMode::Idle).motion,
            MotionMode::Wander
        );
    }

    #[test]
    fn anchorless_warrior_still_flees() {
        // Fleeing is about the target, not home: no anchor needed.
        let hurt = DecisionContext {
            target: Some(visible_target(30.0)),
            slot_ready: mainhand_ready(),
            health_fraction: 0.1,
            ..context()
        };
        let decision = decide(&WARRIOR, &hurt, MotionMode::Chase);
        assert_eq!(decision.motion, MotionMode::Flee);
    }

    #[test]
    fn warrior_strafes_while_sword_ready() {
        let ctx = DecisionContext {
            target: Some(visible_target(30.0)),
            slot_ready: mainhand_ready(),
            ..context()
        };
        let decision = decide(&WARRIOR, &ctx, MotionMode::Idle);
        assert_eq!(decision.motion, MotionMode::Strafe);
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn warrior_holds_while_sword_cooling() {
        let ctx = DecisionContext {
            target: Some(visible_target(30.0)),
            slot_ready: [false; EquipmentSlot::ALL.len()],
            ..context()
        };
        let decision = decide(&WARRIOR, &ctx, MotionMode::Strafe);
        assert_eq!(
            decision.motion,
            MotionMode::KeepDistance {
                min: 0.0,
                max: MELEE_RANGE
            }
        );
        assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn warrior_drinks_only_once_clear_of_threat() {
        // Badly hurt with the target still on top of it: keep fleeing,
        // do not drink mid-melee.
        let pressured = DecisionContext {
            target: Some(visible_target(30.0)),
            health_fraction: 0.1,
            ..context()
        };
        let decision = decide(&WARRIOR, &pressured, MotionMode::Idle);
        assert!(!decision.wants_heal);
        // Same wounds once the flee has opened distance: drink.
        let clear = DecisionContext {
            target: Some(visible_target(HEAL_SAFE_DISTANCE + 10.0)),
            health_fraction: 0.1,
            ..context()
        };
        let decision = decide(&WARRIOR, &clear, MotionMode::Idle);
        assert!(decision.wants_heal);
        // Healthy at any distance: no drink.
        let healthy = DecisionContext {
            target: Some(visible_target(HEAL_SAFE_DISTANCE + 10.0)),
            health_fraction: 1.0,
            ..context()
        };
        let decision = decide(&WARRIOR, &healthy, MotionMode::Idle);
        assert!(!decision.wants_heal);
    }
}
