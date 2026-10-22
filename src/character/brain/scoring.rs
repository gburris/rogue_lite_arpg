//! Scoring math for the utility brains. [`decide`] prices every option,
//! arbitration is two-level: groups compete on merit before options compete
//! within the winning group, so calm stickiness cannot suppress a reactive move.

use super::model::{
    BandEdge, BehaviorGroup, Brain, BrainDecision, CommitWindow, Consideration, DecisionContext,
    Gate, MotionMode, ScoredOption,
};
use crate::items::prelude::EquipmentSlot;

/// Selects the best option index. Zero or negative scores are not selectable;
/// ties go to the first definition.
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

/// Final bounded score for one option. Gates use exit thresholds when current.
pub fn score_option<T>(
    option: &ScoredOption<T>,
    ctx: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    if !option.priority.is_finite() || option.priority < 0.0 {
        return None;
    }

    for gate in option.gates {
        if !pass_gate(gate, ctx, is_current) {
            return None;
        }
    }

    // IAUS compensation: the geometric mean over considerations keeps options
    // with different consideration counts on a comparable scale, while a zero
    // anywhere still vetoes. Options without considerations score priority.
    let mut product = 1.0;
    let mut count = 0;
    for consideration in option.considerations {
        product *= score_consideration(consideration, ctx)?;
        count += 1;
    }
    let mut score = option.priority;
    if count > 0 {
        score *= product.powf(1.0 / (count as f32));
    }

    (score.is_finite() && score >= 0.0).then_some(score)
}

fn pass_gate(gate: &Gate, ctx: &DecisionContext, is_current: bool) -> bool {
    let Some(distance) = ctx.target.map(|target| target.distance) else {
        return true;
    };
    let Some(band) = ctx.strafe_band else {
        return true;
    };
    if !distance.is_finite() {
        return false;
    }
    let edge = match gate.edge {
        BandEdge::Min => band.min(),
        BandEdge::Max => band.max(),
    };
    let offset = distance - edge;
    let threshold = if is_current {
        gate.exit_offset
    } else {
        gate.enter_offset
    };
    if gate.exit_offset < gate.enter_offset {
        offset >= threshold
    } else {
        offset <= threshold
    }
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
    let wants_breathe = select_best(brain.breath_options, ctx, None).is_some();

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
        let group = pick_group(brain.motion_options, ctx, current_index);
        select_best_where(brain.motion_options, ctx, current_index, |mode| {
            mode.group() == group
        })
        .and_then(|index| brain.motion_options.get(index))
    };

    BrainDecision {
        motion: selected.map_or(current, |option| option.value),
        slot_intents,
        wants_heal,
        wants_breathe,
    }
}

fn pick_group(
    options: &[ScoredOption<MotionMode>],
    ctx: &DecisionContext,
    current: Option<usize>,
) -> BehaviorGroup {
    let mut scores = [
        (BehaviorGroup::Aggro, 0.0f32),
        (BehaviorGroup::Return, 0.0f32),
        (BehaviorGroup::Passive, 0.0f32),
    ];
    for (index, option) in options.iter().enumerate() {
        if let Some(entry) = scores
            .iter_mut()
            .find(|(group, _)| *group == option.value.group())
            && let Some(score) = score_option(option, ctx, current == Some(index))
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

pub(crate) const MIN_DWELL_SECS: f32 = 0.3;
pub(crate) const CHALLENGER_MARGIN: f32 = 0.25;

/// Temporal arbiter over the memoryless [`decide`] proposal. Group changes go
/// through at once; only moves within the winning group face the dwell and
/// the challenger margin. Ineligible incumbents never hold.
pub fn arbitrate(
    current: MotionMode,
    proposed: MotionMode,
    current_score: Option<f32>,
    proposed_score: Option<f32>,
    dwell_remaining: f32,
) -> MotionMode {
    if proposed == current {
        return current;
    }
    if proposed.group() != current.group() {
        return proposed;
    }
    // A called swoop, kite break, fire burst, or ice slide is a committed move,
    // not a sustained mode: it ends itself by clearing its call latch and the
    // lottery (or the swing) spaces attempts, so it cannot flicker. Holding it
    // for the intra-group dwell would only burn the call while boundary churn
    // keeps the dwell fresh.
    if proposed.is_commitment() && proposed_score.is_some_and(|score| score > 0.0) {
        return proposed;
    }
    let Some(current_score) = current_score.filter(|score| *score > 0.0) else {
        return proposed;
    };
    let Some(proposed_score) = proposed_score.filter(|score| *score > 0.0) else {
        return current;
    };
    if dwell_remaining > 0.0 {
        return current;
    }
    if proposed_score > current_score + CHALLENGER_MARGIN {
        proposed
    } else {
        current
    }
}

/// Runs one scored tick and lets the arbiter accept or reject the proposal.
pub fn decide_with_arbiter(
    brain: &Brain,
    ctx: &DecisionContext,
    current: MotionMode,
    dwell_remaining: f32,
) -> (BrainDecision, MotionMode) {
    let decision = decide(brain, ctx, current);
    let current_score = score_mode(brain, ctx, current, true);
    let proposed_score = score_mode(brain, ctx, decision.motion, false);
    let accepted = arbitrate(
        current,
        decision.motion,
        current_score,
        proposed_score,
        dwell_remaining,
    );
    (decision, accepted)
}

fn score_mode(
    brain: &Brain,
    ctx: &DecisionContext,
    mode: MotionMode,
    is_current: bool,
) -> Option<f32> {
    brain
        .motion_options
        .iter()
        .find(|option| option.value == mode)
        .and_then(|option| score_option(option, ctx, is_current))
}

/// Commit window of the option driving `mode`, if any.
pub fn commit_for(brain: &Brain, mode: MotionMode) -> Option<CommitWindow> {
    brain
        .motion_options
        .iter()
        .find(|option| option.value == mode)
        .and_then(|option| option.commit)
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
        validate_option(option).map_err(|error| format!("heal option {option_index}: {error}"))?;
    }
    for (option_index, option) in brain.breath_options.iter().enumerate() {
        validate_option(option)
            .map_err(|error| format!("breath option {option_index}: {error}"))?;
    }

    Ok(())
}

fn validate_option<T>(option: &ScoredOption<T>) -> Result<(), String> {
    if !option.priority.is_finite() || option.priority <= 0.0 {
        return Err("priority must be finite and positive".to_string());
    }

    if let Some(window) = option.commit
        && (!window.min().is_finite()
            || !window.max().is_finite()
            || window.min() < 0.0
            || window.max() < 0.0
            || window.min() > window.max())
    {
        return Err("commit bounds must be finite, non-negative, and ordered".to_string());
    }

    for consideration in option.considerations {
        let range = consideration.range;
        if !range.min().is_finite() || !range.max().is_finite() || range.min() >= range.max() {
            return Err(format!(
                "consideration '{}' has an invalid range",
                consideration.name
            ));
        }
    }

    for gate in option.gates {
        if !gate.enter_offset.is_finite() || !gate.exit_offset.is_finite() {
            return Err(format!(
                "gate '{}' needs finite enter/exit offsets",
                gate.name
            ));
        }
        if (gate.enter_offset - gate.exit_offset).abs() <= f32::EPSILON {
            return Err(format!(
                "gate '{}' needs distinct enter/exit offsets",
                gate.name
            ));
        }
    }

    Ok(())
}

fn validate_motion(motion: MotionMode) -> Result<(), String> {
    if let MotionMode::KeepDistance(band) = motion
        && (!band.min().is_finite() || !band.max().is_finite() || band.min() >= band.max())
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
    let normalized = consideration.range.normalize(raw)?;
    Some(consideration.curve.apply(normalized))
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::super::brains::{BAT, FIRE_MAGE, ICE_MAGE, VILLAGER, WARRIOR};
    use super::super::considerations::{HAS_TARGET, MAINHAND_READY};
    use super::super::model::{
        CAST_BAND, CAST_CORE_BAND, HEAL_SAFE_DISTANCE, MELEE_BAND, TargetView,
    };
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
            bearing: Vec2::X,
            velocity: Vec2::ZERO,
            tangential_velocity: 0.0,
        }
    }

    fn warrior_context() -> DecisionContext {
        DecisionContext {
            target: Some(visible_target(30.0)),
            slot_ready: mainhand_ready(),
            strafe_band: Some(MELEE_BAND),
            ..context()
        }
    }

    fn mage_context(distance: f32) -> DecisionContext {
        DecisionContext {
            target: Some(visible_target(distance)),
            slot_ready: mainhand_ready(),
            strafe_band: Some(CAST_BAND),
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
    fn warrior_in_reach_holds_and_requests_mainhand() {
        let ctx = warrior_context();
        let decision = decide(&WARRIOR, &ctx, MotionMode::Chase);

        assert_eq!(decision.motion, MotionMode::KeepDistance(MELEE_BAND));
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

        assert_eq!(decision.motion, MotionMode::KeepDistance(MELEE_BAND));
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
            ..warrior_context()
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
        assert_eq!(decision.motion, MotionMode::KeepDistance(MELEE_BAND));
    }

    #[test]
    fn fire_mage_orbits_and_fires() {
        let ctx = mage_context(250.0);
        let decision = decide(&FIRE_MAGE, &ctx, MotionMode::Idle);

        assert_eq!(decision.motion, MotionMode::Strafe);
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn ice_mage_plants_while_cooling_down() {
        // Readiness gates the trigger finger, never the stance: the cooldown
        // must not make the planted ice mage twitch on its firing lane.
        let ctx = DecisionContext {
            slot_ready: [false; EquipmentSlot::ALL.len()],
            ..mage_context(250.0)
        };
        let decision = decide(&ICE_MAGE, &ctx, MotionMode::Hold);

        assert_eq!(decision.motion, MotionMode::Hold);
        assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn both_mages_chase_beyond_their_band() {
        let ctx = mage_context(CAST_BAND.max() + 50.0);
        for brain in [&FIRE_MAGE, &ICE_MAGE] {
            let decision = decide(brain, &ctx, MotionMode::KeepDistance(CAST_BAND));
            assert_eq!(decision.motion, MotionMode::Chase);
        }
    }

    #[test]
    fn bat_strafes_in_band_and_breathes() {
        let in_band = DecisionContext {
            target: Some(visible_target(CAST_BAND.max() - 50.0)),
            strafe_band: Some(CAST_BAND),
            ..context()
        };
        let decision = decide(&BAT, &in_band, MotionMode::Idle);
        assert_eq!(decision.motion, MotionMode::Strafe);
        assert!(decision.wants_breathe);

        let too_far = DecisionContext {
            target: Some(visible_target(CAST_BAND.max() + 50.0)),
            ..in_band
        };
        let decision = decide(&BAT, &too_far, MotionMode::Strafe);
        assert_eq!(decision.motion, MotionMode::Chase);
        assert!(!decision.wants_breathe);

        let too_close = DecisionContext {
            target: Some(visible_target(CAST_BAND.min() - 50.0)),
            ..in_band
        };
        let decision = decide(&BAT, &too_close, MotionMode::Idle);
        assert_eq!(decision.motion, MotionMode::KeepDistance(CAST_CORE_BAND));
    }

    #[test]
    fn bat_swoops_only_when_called() {
        let circling = DecisionContext {
            target: Some(visible_target(CAST_BAND.max() - 50.0)),
            strafe_band: Some(CAST_BAND),
            swoop_called: true,
            ..context()
        };
        assert_eq!(
            decide(&BAT, &circling, MotionMode::Strafe).motion,
            MotionMode::Swoop
        );
        // No call, no dive: the circle holds.
        let uncalled = DecisionContext {
            swoop_called: false,
            ..circling
        };
        assert_eq!(
            decide(&BAT, &uncalled, MotionMode::Strafe).motion,
            MotionMode::Strafe
        );
        // A cleared call (the dive ended) hands an incumbent dive back at
        // once instead of re-firing on a stale latch.
        assert_eq!(
            decide(&BAT, &uncalled, MotionMode::Swoop).motion,
            MotionMode::Strafe
        );
        // A called dive stays eligible as it closes past the inner band edge:
        // gating it by band distance would cancel the dive mid-flight.
        let closed_in = DecisionContext {
            target: Some(visible_target(CAST_BAND.min() - 100.0)),
            ..circling
        };
        assert_eq!(
            decide(&BAT, &closed_in, MotionMode::Swoop).motion,
            MotionMode::Swoop
        );
    }

    #[test]
    fn mage_without_line_of_sight_does_not_request_mainhand() {
        let ctx = DecisionContext {
            target: Some(TargetView {
                distance: 250.0,
                line_of_sight: false,
                bearing: Vec2::X,
                velocity: Vec2::ZERO,
                tangential_velocity: 0.0,
            }),
            slot_ready: mainhand_ready(),
            ..context()
        };
        for brain in [&FIRE_MAGE, &ICE_MAGE] {
            let decision = decide(brain, &ctx, MotionMode::Idle);
            assert!(!decision.slot_intents[EquipmentSlot::Mainhand as usize]);
        }
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
            anchor_arrived: Some(false),
            ..warrior_context()
        };
        assert_eq!(
            decide(&WARRIOR, &contact, MotionMode::ReturnToAnchor).motion,
            MotionMode::Chase
        );
    }

    #[test]
    fn anchorless_warrior_still_flees() {
        // Fleeing is about the target, not home: no anchor needed.
        let hurt = DecisionContext {
            health_fraction: 0.1,
            ..warrior_context()
        };
        let decision = decide(&WARRIOR, &hurt, MotionMode::Chase);
        assert_eq!(decision.motion, MotionMode::Flee);
    }

    #[test]
    fn warrior_holds_reach_while_sword_ready() {
        let ctx = warrior_context();
        let decision = decide(&WARRIOR, &ctx, MotionMode::Idle);
        assert_eq!(decision.motion, MotionMode::KeepDistance(MELEE_BAND));
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
    }

    #[test]
    fn warrior_holds_while_sword_cooling() {
        let ctx = DecisionContext {
            slot_ready: [false; EquipmentSlot::ALL.len()],
            ..warrior_context()
        };
        let decision = decide(&WARRIOR, &ctx, MotionMode::KeepDistance(MELEE_BAND));
        assert_eq!(decision.motion, MotionMode::KeepDistance(MELEE_BAND));
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

    const TEST_MELEE_HOLD_GATE: Gate = Gate::new("inside melee edge", BandEdge::Max, 0.0, 4.0);

    /// No chase option: group scoring has to keep the incumbent's aggro group
    /// alive while it sits inside its stay band but outside its enter band.
    static STRAFE_KEEP_ONLY: &[ScoredOption<MotionMode>] = &[
        ScoredOption {
            value: MotionMode::Strafe,
            priority: 5.5,
            commit: None,
            considerations: &[HAS_TARGET, MAINHAND_READY],
            gates: &[TEST_MELEE_HOLD_GATE],
        },
        ScoredOption {
            value: MotionMode::KeepDistance(MELEE_BAND),
            priority: 5.0,
            commit: None,
            considerations: &[HAS_TARGET],
            gates: &[TEST_MELEE_HOLD_GATE],
        },
    ];

    #[test]
    fn group_scoring_keeps_an_incumbent_inside_its_stay_band() {
        let ctx = DecisionContext {
            target: Some(visible_target(66.0)),
            slot_ready: mainhand_ready(),
            strafe_band: Some(MELEE_BAND),
            ..context()
        };

        // 66 is past the 64 px enter edge but inside the keeper's 68 px exit.
        assert_eq!(
            pick_group(STRAFE_KEEP_ONLY, &ctx, Some(1)),
            BehaviorGroup::Aggro
        );
        // As a challenger neither option can enter, so there is no merit.
        assert_eq!(
            pick_group(STRAFE_KEEP_ONLY, &ctx, None),
            BehaviorGroup::Passive
        );
    }

    #[test]
    fn approach_and_hold_share_the_melee_deadband() {
        let near_edge = DecisionContext {
            target: Some(visible_target(66.0)),
            ..warrior_context()
        };
        let keeper = MotionMode::KeepDistance(MELEE_BAND);

        // Whichever option reaches the shared band first persists in it.
        assert_eq!(
            decide_with_arbiter(&WARRIOR, &near_edge, MotionMode::Chase, 0.0).1,
            MotionMode::Chase
        );
        assert_eq!(
            decide_with_arbiter(&WARRIOR, &near_edge, keeper, 0.0).1,
            keeper
        );

        // Past the exit edge the approach option takes over.
        let beyond = DecisionContext {
            target: Some(visible_target(69.0)),
            ..near_edge
        };
        assert_eq!(
            decide_with_arbiter(&WARRIOR, &beyond, keeper, 0.0).1,
            MotionMode::Chase
        );
    }

    #[test]
    fn approach_and_hold_share_the_cast_deadband() {
        let near_edge = DecisionContext {
            target: Some(visible_target(CAST_BAND.max() - 1.0)),
            strafe_band: Some(CAST_BAND),
            ..context()
        };

        assert_eq!(
            decide_with_arbiter(&BAT, &near_edge, MotionMode::Chase, 0.0).1,
            MotionMode::Chase
        );
        assert_eq!(
            decide_with_arbiter(&BAT, &near_edge, MotionMode::Strafe, 0.0).1,
            MotionMode::Strafe
        );

        let beyond = DecisionContext {
            target: Some(visible_target(CAST_BAND.max() + 6.0)),
            ..near_edge
        };
        assert_eq!(
            decide_with_arbiter(&BAT, &beyond, MotionMode::Strafe, 0.0).1,
            MotionMode::Chase
        );
    }

    #[test]
    fn ice_mage_holds_its_lane_inside_the_band() {
        let planted = MotionMode::Hold;

        // Inside the inner edge, at the edge, and near the outer edge the ice
        // mage keeps its firing stance instead of switching into an orbit.
        let (_, held) = decide_with_arbiter(&ICE_MAGE, &mage_context(200.0), planted, 0.0);
        assert_eq!(held, planted);
        let (_, still_held) = decide_with_arbiter(&ICE_MAGE, &mage_context(220.0), held, 0.0);
        assert_eq!(still_held, planted);
        let (_, at_outer_edge) =
            decide_with_arbiter(&ICE_MAGE, &mage_context(350.0), still_held, 0.0);
        assert_eq!(at_outer_edge, planted);
    }

    #[test]
    fn arbiter_holds_during_the_dwell() {
        let keeper = MotionMode::KeepDistance(CAST_CORE_BAND);
        assert_eq!(
            arbitrate(MotionMode::Strafe, keeper, Some(5.5), Some(9.0), 0.1),
            MotionMode::Strafe
        );
    }
    #[test]
    fn arbiter_releases_a_called_swoop_through_the_dwell() {
        assert_eq!(
            arbitrate(
                MotionMode::Strafe,
                MotionMode::Swoop,
                Some(5.5),
                Some(6.0),
                0.2
            ),
            MotionMode::Swoop
        );
    }

    #[test]
    fn arbiter_ignores_a_zero_score_incumbent() {
        let keeper = MotionMode::KeepDistance(CAST_CORE_BAND);
        assert_eq!(
            arbitrate(keeper, MotionMode::Strafe, Some(0.0), Some(5.5), 1.0),
            MotionMode::Strafe
        );
        assert_eq!(
            arbitrate(keeper, MotionMode::Strafe, None, Some(5.5), 1.0),
            MotionMode::Strafe
        );
    }

    #[test]
    fn arbiter_requires_a_real_margin() {
        let keeper = MotionMode::KeepDistance(CAST_CORE_BAND);
        assert_eq!(
            arbitrate(keeper, MotionMode::Strafe, Some(5.0), Some(5.2), 0.0),
            keeper
        );
        assert_eq!(
            arbitrate(keeper, MotionMode::Strafe, Some(5.0), Some(5.3), 0.0),
            MotionMode::Strafe
        );
    }

    #[test]
    fn arbiter_accepts_group_changes_immediately() {
        assert_eq!(
            arbitrate(
                MotionMode::Idle,
                MotionMode::Chase,
                Some(0.0),
                Some(1.0),
                10.0
            ),
            MotionMode::Chase
        );
    }
}
