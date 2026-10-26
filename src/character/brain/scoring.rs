//! Scoring math for the utility brains. Every decision scores its weight times
//! the geometric mean of its considerations. Behaviors then take the motion
//! and action slots greedily from the highest score down; a behavior that
//! already holds a slot scores with [`MOMENTUM`]. A slot nothing scores for
//! stays empty.

use super::model::{BrainDefinition, Consideration, Decision, DecisionContext, Motion};

/// Score multiplier a behavior keeps over challengers while it holds a slot, so
/// a near-tie does not flip the body back and forth between ticks.
pub(crate) const MOMENTUM: f32 = 1.25;

/// Most behaviors one brain definition may list, so scoring needs no allocation.
pub(crate) const MAX_BEHAVIORS: usize = 16;

/// A behavior holding a slot, by its index in the definition.
#[derive(Clone, Copy, Debug)]
pub struct SlotHolder {
    pub behavior: usize,
    pub locked: bool,
}

/// Who holds each slot going into a decision.
#[derive(Clone, Copy, Debug, Default)]
pub struct SlotHolders {
    pub motion: Option<SlotHolder>,
    pub action: Option<SlotHolder>,
}

/// The behaviors that won a slot this decision, by index. A slot nothing
/// claimed is `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Assignment {
    pub motion: Option<usize>,
    pub action: Option<usize>,
}

/// Final bounded score for one decision. `is_current` switches threshold
/// considerations to their exit side.
pub fn score_decision<T>(
    decision: &Decision<T>,
    decision_context: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // IAUS compensation: the geometric mean over considerations keeps
    // decisions with different consideration counts on a comparable scale,
    // while a zero anywhere still vetoes. No considerations scores the weight.
    let mut product = 1.0;
    let mut count = 0;
    for consideration in decision.considerations {
        product *= score_consideration(consideration, decision_context, is_current)?;
        count += 1;
    }
    let mut score = decision.weight;
    if count > 0 {
        score *= product.powf(1.0 / (count as f32));
    }

    (score.is_finite() && score >= 0.0).then_some(score)
}

/// Fills the free slots. Locked slots keep their holder and are not scored;
/// every other behavior whose claimed slots are all free is scored, then
/// taken from the highest score down whenever its slots are still free (ties go
/// to the earlier behavior). `start` is asked before a winner is taken, and a
/// `false` drops it and moves on to the next best.
pub fn assign_behaviors_to_slots(
    definition: &BrainDefinition,
    decision_context: &DecisionContext,
    holders: SlotHolders,
    mut start: impl FnMut(usize) -> bool,
) -> Assignment {
    let motion_free = holders.motion.is_none_or(|holder| !holder.locked);
    let action_free = holders.action.is_none_or(|holder| !holder.locked);

    let mut scores = [0.0_f32; MAX_BEHAVIORS];
    for (index, decision) in definition.behaviors.iter().enumerate() {
        let behavior = decision.value;
        let blocked = (behavior.motion.is_some() && !motion_free)
            || (behavior.action.is_some() && !action_free);
        let unable = behavior
            .motion
            .is_some_and(|motion| !decision_context.can_start(motion))
            || behavior
                .action
                .is_some_and(|action| !decision_context.can_perform(action));
        if blocked || unable {
            continue;
        }
        let is_current = [holders.motion, holders.action]
            .into_iter()
            .flatten()
            .any(|holder| holder.behavior == index);
        let Some(score) = score_decision(decision, decision_context, is_current) else {
            continue;
        };
        scores[index] = if is_current { score * MOMENTUM } else { score };
    }

    let mut assignment = Assignment::default();
    loop {
        let mut best: Option<usize> = None;
        for (index, decision) in definition.behaviors.iter().enumerate() {
            let behavior = decision.value;
            let taken = (behavior.motion.is_some() && assignment.motion.is_some())
                || (behavior.action.is_some() && assignment.action.is_some());
            if scores[index] <= 0.0 || taken {
                continue;
            }
            if best.is_none_or(|leader| scores[index] > scores[leader]) {
                best = Some(index);
            }
        }
        let Some(index) = best else {
            return assignment;
        };
        if !start(index) {
            scores[index] = 0.0;
            continue;
        }
        let behavior = definition.behaviors[index].value;
        if behavior.motion.is_some() {
            assignment.motion = Some(index);
        }
        if behavior.action.is_some() {
            assignment.action = Some(index);
        }
    }
}

/// Rejects brains that compile but cannot behave: an empty list, a behavior
/// that claims nothing, a lock with no finite part (it could never end), or a
/// finite motion left unlocked (nothing would track its end). Weights,
/// intervals and thresholds are checked by their constructors at compile time.
pub fn validate_brain(brain: &BrainDefinition) -> Result<(), String> {
    if brain.behaviors.is_empty() {
        return Err("behavior list is empty".to_string());
    }
    if brain.behaviors.len() > MAX_BEHAVIORS {
        return Err(format!("more than {MAX_BEHAVIORS} behaviors"));
    }
    for (index, decision) in brain.behaviors.iter().enumerate() {
        let behavior = decision.value;
        if behavior.motion.is_none() && behavior.action.is_none() {
            return Err(format!("behavior {index} claims no slot"));
        }
        // Every action is finite, so any action is a finite part of its lock.
        let finite_motion = behavior.motion.is_some_and(Motion::is_finite);
        if behavior.commit.is_lock() && !finite_motion && behavior.action.is_none() {
            return Err(format!(
                "behavior {index} is locked but has no finite part, so it could never end"
            ));
        }
        if !behavior.commit.is_lock() && finite_motion {
            return Err(format!("behavior {index} has a finite motion but no lock"));
        }
    }
    Ok(())
}

fn score_consideration(
    consideration: &Consideration,
    decision_context: &DecisionContext,
    is_current: bool,
) -> Option<f32> {
    // A missing input (no target, no anchor) vetoes, inverted or not: a fact
    // about something absent cannot be true.
    let Some(raw) = (consideration.extract_raw_input)(decision_context) else {
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
    use super::super::model::{
        Action, CAST_DISTANCE_RANGE, MELEE_DISTANCE_RANGE, TargetView, Tuning,
    };
    use super::*;
    use crate::items::prelude::EquipmentSlot;

    const MAINHAND: Action = Action::Use(EquipmentSlot::Mainhand);

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
        }
    }

    /// A ready mainhand with the brain's equipment reach, as the senses stage it.
    fn armed(distance: f32, brain: &BrainDefinition) -> DecisionContext {
        let mut decision_context = DecisionContext {
            target: Some(visible_target(distance)),
            slot_ready: mainhand_ready(),
            ..context()
        };
        decision_context.action_tuning[MAINHAND.index()] = Some(Tuning {
            reach: brain.equipment_reach,
            ..Tuning::default()
        });
        decision_context
    }

    fn warrior_context() -> DecisionContext {
        armed(30.0, &WARRIOR)
    }

    fn mage_context(distance: f32) -> DecisionContext {
        armed(distance, &FIRE_MAGE)
    }

    /// The motion and action the brain picks while `current` holds the motion slot.
    fn choose(
        brain: &BrainDefinition,
        decision_context: &DecisionContext,
        current: Motion,
    ) -> (Motion, Option<Action>) {
        let holder = brain
            .behaviors
            .iter()
            .position(|decision| decision.value.motion == Some(current))
            .expect("brain lists the current motion");
        let holders = SlotHolders {
            motion: Some(SlotHolder {
                behavior: holder,
                locked: false,
            }),
            action: None,
        };
        let assignment = assign_behaviors_to_slots(brain, decision_context, holders, |_| true);
        let motion = assignment
            .motion
            .and_then(|index| brain.behaviors[index].value.motion)
            .expect("a motion scores");
        let action = assignment
            .action
            .and_then(|index| brain.behaviors[index].value.action);
        (motion, action)
    }

    #[test]
    fn warrior_in_reach_holds_and_requests_mainhand() {
        let decision_context = warrior_context();

        assert_eq!(
            choose(&WARRIOR, &decision_context, Motion::Chase),
            (Motion::KeepDistance(MELEE_DISTANCE_RANGE), Some(MAINHAND))
        );
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
        // Still swinging on the way out: a fighting retreat, not a rout.
        assert_eq!(
            choose(&WARRIOR, &fleeing, Motion::Chase),
            (Motion::Flee, Some(MAINHAND))
        );
        let scratched = DecisionContext {
            health_fraction: 0.5,
            ..fleeing
        };
        assert_eq!(
            choose(&WARRIOR, &scratched, Motion::Chase).0,
            Motion::KeepDistance(MELEE_DISTANCE_RANGE)
        );
    }

    #[test]
    fn fire_mage_stands_to_cast_and_orbits_otherwise() {
        let decision_context = mage_context(250.0);

        assert_eq!(
            choose(&FIRE_MAGE, &decision_context, Motion::Calm),
            (Motion::Idle, Some(MAINHAND))
        );
        let cooling = DecisionContext {
            slot_ready: [false; EquipmentSlot::ALL.len()],
            ..decision_context
        };
        assert_eq!(
            choose(&FIRE_MAGE, &cooling, Motion::Calm),
            (Motion::Strafe(CAST_DISTANCE_RANGE), None)
        );
    }
}
