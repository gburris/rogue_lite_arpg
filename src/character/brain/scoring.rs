//! Scoring math for the utility brains. [`decide`] prices every option,
//! arbitration is two-level: groups compete on merit before options compete
//! within the winning group, so calm stickiness cannot suppress a reactive move.

use super::model::{
    Ability, BandEdge, BehaviorGroup, BrainDecision, BrainDefinition, CommitWindow, Consideration,
    DecisionContext, DistanceBand, Gate, Maneuver, MotionMode, ScoredOption,
};
use crate::items::prelude::EquipmentSlot;

/// Selects the best option index. Zero or negative scores are not selectable;
/// ties go to the first definition.
pub fn select_best<T>(
    options: &[ScoredOption<T>],
    ctx: &DecisionContext,
    band: Option<DistanceBand>,
    current: Option<usize>,
) -> Option<usize> {
    select_best_where(options, ctx, band, current, |_| true)
}

fn select_best_where<T>(
    options: &[ScoredOption<T>],
    ctx: &DecisionContext,
    band: Option<DistanceBand>,
    current: Option<usize>,
    mut include: impl FnMut(&T) -> bool,
) -> Option<usize> {
    let mut best = None;

    for (index, option) in options.iter().enumerate() {
        if !include(&option.value) {
            continue;
        }
        let Some(score) = score_option(option, ctx, band, current == Some(index)) else {
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

/// Final bounded score for one option. Gates measure against `band` and use
/// exit thresholds when current.
pub fn score_option<T>(
    option: &ScoredOption<T>,
    ctx: &DecisionContext,
    band: Option<DistanceBand>,
    is_current: bool,
) -> Option<f32> {
    if !option.priority.is_finite() || option.priority < 0.0 {
        return None;
    }

    for gate in option.gates {
        if !pass_gate(gate, ctx, band, is_current) {
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

fn pass_gate(
    gate: &Gate,
    ctx: &DecisionContext,
    band: Option<DistanceBand>,
    is_current: bool,
) -> bool {
    let Some(distance) = ctx.target.map(|target| target.distance) else {
        return true;
    };
    let Some(band) = band else {
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
pub fn choose_slot(brain: &BrainDefinition, slot: EquipmentSlot, ctx: &DecisionContext) -> bool {
    select_best(brain.slot_options[slot as usize], ctx, brain.band, None).is_some()
}

/// Best ability option per ability. Abilities the entity cannot carry out are
/// never selectable.
fn choose_abilities(brain: &BrainDefinition, ctx: &DecisionContext) -> [bool; Ability::ALL.len()] {
    let mut abilities = [false; Ability::ALL.len()];
    for ability in Ability::ALL {
        let index = ability as usize;
        abilities[index] = ctx.self_state.ability_tuning[index].is_some()
            && select_best(brain.ability_options[index], ctx, brain.band, None).is_some();
    }
    abilities
}

/// Best maneuver ready to start. A running maneuver is exclusive, so nothing
/// else is proposed while one is active.
fn choose_maneuver(brain: &BrainDefinition, ctx: &DecisionContext) -> Option<Maneuver> {
    if ctx.self_state.active_maneuver.is_some() {
        return None;
    }
    select_best_where(brain.maneuver_options, ctx, brain.band, None, |maneuver| {
        ctx.self_state.maneuver_ready[*maneuver as usize]
    })
    .map(|index| brain.maneuver_options[index].value)
}

/// Produces the complete locomotion, maneuver, slot and ability decision for one tick.
pub fn decide(brain: &BrainDefinition, ctx: &DecisionContext, current: MotionMode) -> BrainDecision {
    let mut slot_intents = [false; EquipmentSlot::ALL.len()];
    for slot in EquipmentSlot::ALL {
        slot_intents[slot as usize] = choose_slot(brain, slot, ctx);
    }

    let current_index = brain
        .motion_options
        .iter()
        .position(|option| option.value == current);

    // Once a character starts returning to its anchor, keep the mode held
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
        let group = pick_group(brain.motion_options, ctx, brain.band, current_index);
        select_best_where(brain.motion_options, ctx, brain.band, current_index, |mode| {
            mode.group() == group
        })
        .and_then(|index| brain.motion_options.get(index))
    };

    BrainDecision {
        motion: selected.map_or(current, |option| option.value),
        maneuver: choose_maneuver(brain, ctx),
        slot_intents,
        abilities: choose_abilities(brain, ctx),
    }
}

fn pick_group(
    options: &[ScoredOption<MotionMode>],
    ctx: &DecisionContext,
    band: Option<DistanceBand>,
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
            && let Some(score) = score_option(option, ctx, band, current == Some(index))
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
    brain: &BrainDefinition,
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
    brain: &BrainDefinition,
    ctx: &DecisionContext,
    mode: MotionMode,
    is_current: bool,
) -> Option<f32> {
    brain
        .motion_options
        .iter()
        .find(|option| option.value == mode)
        .and_then(|option| score_option(option, ctx, brain.band, is_current))
}

/// Commit window of the option driving `mode`, if any.
pub fn commit_for(brain: &BrainDefinition, mode: MotionMode) -> Option<CommitWindow> {
    brain
        .motion_options
        .iter()
        .find(|option| option.value == mode)
        .and_then(|option| option.commit)
}

/// Rejects malformed brains before they can silently produce nonsense.
pub fn validate_brain(brain: &BrainDefinition) -> Result<(), String> {
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

    for (index, option) in brain.maneuver_options.iter().enumerate() {
        validate_option(option).map_err(|error| format!("maneuver option {index}: {error}"))?;

        if brain.maneuver_options[..index]
            .iter()
            .any(|earlier| earlier.value == option.value)
        {
            return Err(format!("maneuver option {index} duplicates an earlier maneuver"));
        }
    }

    for (slot_index, options) in brain.slot_options.iter().enumerate() {
        for (option_index, option) in options.iter().enumerate() {
            validate_option(option)
                .map_err(|error| format!("slot {slot_index} option {option_index}: {error}"))?;
        }
    }
    for (ability_index, options) in brain.ability_options.iter().enumerate() {
        for (option_index, option) in options.iter().enumerate() {
            validate_option(option)
                .map_err(|error| format!("ability {ability_index} option {option_index}: {error}"))?;
        }
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

    use super::super::brains::{FIRE_MAGE, WARRIOR};
    use super::super::model::{MELEE_BAND, TargetView};
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
            ..context()
        }
    }

    fn mage_context(distance: f32) -> DecisionContext {
        DecisionContext {
            target: Some(visible_target(distance)),
            slot_ready: mainhand_ready(),
            ..context()
        }
    }

    #[test]
    fn warrior_in_reach_holds_and_requests_mainhand() {
        let ctx = warrior_context();
        let decision = decide(&WARRIOR, &ctx, MotionMode::Chase);

        assert_eq!(decision.motion, MotionMode::KeepDistance(MELEE_BAND));
        assert!(decision.slot_intents[EquipmentSlot::Mainhand as usize]);
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
}
