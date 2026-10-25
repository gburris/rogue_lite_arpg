//! Scoring math for the utility brains. Every decision scores its weight times
//! the geometric mean of its considerations. The movement layer takes the
//! single best decision, with momentum on the current mode; the action layer
//! fires every action with a decision that clears [`ACTION_THRESHOLD`].

use super::model::{
    Action, BrainDefinition, Consideration, Decision, DecisionContext, MotionMode, Movement,
};

/// Score multiplier the current mode keeps over challengers, so a near-tie
/// does not flip the body back and forth between ticks.
pub(crate) const MOMENTUM: f32 = 1.25;

/// Score an action decision must reach to fire. The geometric mean softens a
/// single graded consideration (with `n` considerations it must reach about
/// `0.9^n`), so the bar sits high enough that graded inputs like cast quality
/// still shape when an action fires.
pub(crate) const ACTION_THRESHOLD: f32 = 0.9;

/// Final bounded score for one decision. `is_current` switches threshold
/// considerations to their exit side.
pub fn score_decision<T>(
    decision: &Decision<T>,
    ctx: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // IAUS compensation: the geometric mean over considerations keeps
    // decisions with different consideration counts on a comparable scale,
    // while a zero anywhere still vetoes. No considerations scores the weight.
    let mut product = 1.0;
    let mut count = 0;
    for consideration in decision.considerations {
        product *= score_consideration(consideration, ctx, is_current)?;
        count += 1;
    }
    let mut score = decision.weight;
    if count > 0 {
        score *= product.powf(1.0 / (count as f32));
    }

    (score.is_finite() && score >= 0.0).then_some(score)
}

/// Best movement the body can start, or `None` when nothing scores. The
/// current mode scores with [`MOMENTUM`]; ties go to the earlier decision.
pub fn choose_movement(
    brain: &BrainDefinition,
    ctx: &DecisionContext,
    current: MotionMode,
    mut can_start: impl FnMut(Movement) -> bool,
) -> Option<Movement> {
    let mut best: Option<(Movement, f32)> = None;
    for decision in brain.movement {
        if !can_start(decision.value) {
            continue;
        }
        let is_current = decision.value == Movement::Mode(current);
        let Some(mut score) = score_decision(decision, ctx, is_current) else {
            continue;
        };
        if is_current {
            score *= MOMENTUM;
        }
        if score <= 0.0 {
            continue;
        }
        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((decision.value, score)),
        }
    }
    best.map(|(movement, _)| movement)
}

/// Every action the body can perform with a decision at or above [`ACTION_THRESHOLD`].
pub fn choose_actions(brain: &BrainDefinition, ctx: &DecisionContext) -> [bool; Action::COUNT] {
    let mut actions = [false; Action::COUNT];
    for decision in brain.actions {
        let index = decision.value.index();
        if actions[index] || !ctx.can_perform(decision.value) {
            continue;
        }
        actions[index] =
            score_decision(decision, ctx, false).is_some_and(|score| score >= ACTION_THRESHOLD);
    }
    actions
}

/// Rejects brains that compile but cannot behave: an empty movement layer or
/// a movement listed twice. Weights, intervals and thresholds are checked by
/// their constructors at compile time.
pub fn validate_brain(brain: &BrainDefinition) -> Result<(), String> {
    if brain.movement.is_empty() {
        return Err("movement list is empty".to_string());
    }
    for (index, decision) in brain.movement.iter().enumerate() {
        if brain.movement[..index]
            .iter()
            .any(|earlier| earlier.value == decision.value)
        {
            return Err(format!(
                "movement decision {index} duplicates an earlier movement"
            ));
        }
    }
    Ok(())
}

fn score_consideration(
    consideration: &Consideration,
    ctx: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // A missing input (no target, no anchor) vetoes, inverted or not: a fact
    // about something absent cannot be true.
    let Some(raw) = (consideration.read)(ctx) else {
        return Some(0.0);
    };
    let score = consideration.response.score(raw, is_current)?;
    Some(if consideration.inverted {
        1.0 - score
    } else {
        score
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::super::brains::{FIRE_MAGE, WARRIOR};
    use super::super::model::{CAST_BAND, MELEE_BAND, TargetView, Tuning};
    use super::*;
    use crate::items::prelude::EquipmentSlot;

    const MAINHAND: usize = Action::Use(EquipmentSlot::Mainhand).index();

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

    /// A ready mainhand with the brain's equipment reach, as the senses stage it.
    fn armed(distance: f32, brain: &BrainDefinition) -> DecisionContext {
        let mut ctx = DecisionContext {
            target: Some(visible_target(distance)),
            slot_ready: mainhand_ready(),
            ..context()
        };
        ctx.action_tuning[MAINHAND] = Some(Tuning {
            reach: brain.equipment_reach,
            ..Tuning::default()
        });
        ctx
    }

    fn warrior_context() -> DecisionContext {
        armed(30.0, &WARRIOR)
    }

    fn mage_context(distance: f32) -> DecisionContext {
        armed(distance, &FIRE_MAGE)
    }

    fn movement(brain: &BrainDefinition, ctx: &DecisionContext, current: MotionMode) -> Movement {
        choose_movement(brain, ctx, current, |movement| ctx.can_start(movement))
            .expect("a movement scores")
    }

    #[test]
    fn warrior_in_reach_holds_and_requests_mainhand() {
        let ctx = warrior_context();

        assert_eq!(
            movement(&WARRIOR, &ctx, MotionMode::Chase),
            Movement::Mode(MotionMode::KeepDistance(MELEE_BAND))
        );
        assert!(choose_actions(&WARRIOR, &ctx)[MAINHAND]);
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
        assert_eq!(
            movement(&WARRIOR, &fleeing, MotionMode::Chase),
            Movement::Mode(MotionMode::Flee)
        );
        // Still swinging on the way out: a fighting retreat, not a rout.
        assert!(choose_actions(&WARRIOR, &fleeing)[MAINHAND]);
        let scratched = DecisionContext {
            health_fraction: 0.5,
            ..fleeing
        };
        assert_eq!(
            movement(&WARRIOR, &scratched, MotionMode::Chase),
            Movement::Mode(MotionMode::KeepDistance(MELEE_BAND))
        );
    }

    #[test]
    fn fire_mage_orbits_and_fires() {
        let ctx = mage_context(250.0);

        assert_eq!(
            movement(&FIRE_MAGE, &ctx, MotionMode::Calm),
            Movement::Mode(MotionMode::Strafe(CAST_BAND))
        );
        assert!(choose_actions(&FIRE_MAGE, &ctx)[MAINHAND]);
    }
}
