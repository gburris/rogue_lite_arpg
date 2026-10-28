//! The decision pipeline: observe the world, drop a stale committed behavior,
//! score, and commit. This is the only code that changes which behaviors a
//! [`Brain`] holds.

use bevy::prelude::*;

use super::capability::{
    Capabilities, CapabilityComponents, MotionActionTable, build_capabilities,
};
use super::engine::{Behavior, assign_behaviors_to_slots, explain_behavior};
use super::motion::MotionFinished;
use super::perception::{
    DecisionContext, ObservedComponents, ObservedComponentsItem, WorldUseChecks,
    observe_world_facts,
};
use super::{
    Action, ActionFinished, ActionOutcome, Brain, BrainRng, DECISION_PERIOD, ScoreTrace,
};

type DecidingBrains<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut Brain,
        &'static mut BrainRng,
        ObservedComponents,
        CapabilityComponents,
        Option<&'static mut ScoreTrace>,
    ),
>;

pub(super) fn release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, observed, components, mut score_trace)) =
        brains.get_mut(finished.entity)
    else {
        return;
    };
    let capabilities = build_capabilities(&components, &use_checks.equipment);
    let cooldown = capabilities[finished.motion].map_or(0.0, |tuning| tuning.cooldown);
    brain.start_cooldown(finished.motion, now + cooldown);
    if brain.finish_committed_behavior(finished.epoch) {
        redecide_now(
            &mut brain,
            &mut rng,
            &observed,
            &capabilities,
            &use_checks,
            score_trace.as_deref_mut(),
            now,
        );
    }
}

/// Applies an [`ActionFinished`], current or stale. An ability starts its
/// cooldown even if it never started, so the next decision can't pick it
/// straight back.
pub(super) fn finish_action(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, observed, components, mut score_trace)) =
        brains.get_mut(finished.entity)
    else {
        return;
    };
    let capabilities = build_capabilities(&components, &use_checks.equipment);
    match finished.action {
        Action::Use(slot)
            if slot.is_instant_use() && finished.outcome == ActionOutcome::Completed =>
        {
            brain.attack_finish_pending = true;
        }
        Action::Ability(ability) => {
            let cooldown = capabilities[ability].map_or(0.0, |tuning| tuning.cooldown);
            brain.start_cooldown(ability, now + cooldown);
        }
        // A held or unstarted use is no attack, and nothing reports an idle
        // action's finish.
        Action::Use(_) | Action::Idle => {}
    }
    if brain.finish_committed_behavior(finished.epoch) {
        redecide_now(
            &mut brain,
            &mut rng,
            &observed,
            &capabilities,
            &use_checks,
            score_trace.as_deref_mut(),
            now,
        );
    }
}

/// Decides for every due brain. Each moves to its next period boundary after
/// now, keeping its phase without a burst of catch-up decisions after a long frame.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, observed, components, mut score_trace) in &mut brains {
        if now < brain.next_decision_at {
            continue;
        }
        let periods_passed = ((now - brain.next_decision_at) / DECISION_PERIOD).floor() + 1.0;
        brain.next_decision_at += periods_passed * DECISION_PERIOD;
        // Rounding near an exact multiple can land the boundary on `now`.
        if brain.next_decision_at <= now {
            brain.next_decision_at += DECISION_PERIOD;
        }
        let capabilities = build_capabilities(&components, &use_checks.equipment);
        redecide_now(
            &mut brain,
            &mut rng,
            &observed,
            &capabilities,
            &use_checks,
            score_trace.as_deref_mut(),
            now,
        );
    }
}

fn redecide_now(
    brain: &mut Brain,
    rng: &mut BrainRng,
    observed: &ObservedComponentsItem,
    capabilities: &Capabilities,
    use_checks: &WorldUseChecks,
    score_trace: Option<&mut ScoreTrace>,
    now: f32,
) {
    let (observed_context, world_allows_use) =
        observe_world_facts(observed, capabilities, use_checks, now);
    decide_slot_assignments(
        brain,
        rng,
        observed_context,
        &world_allows_use,
        capabilities,
        score_trace,
        now,
    );
}

/// Drops committed behaviors that lost their target, were stunned or timed
/// out, then fills the slots. A winning finite
/// motion rolls its start chance; a miss waits out its cooldown and the next
/// best behavior takes the slot.
fn decide_slot_assignments(
    brain: &mut Brain,
    rng: &mut BrainRng,
    observed_context: DecisionContext,
    world_allows_use: &MotionActionTable<bool>,
    capabilities: &Capabilities,
    score_trace: Option<&mut ScoreTrace>,
    now: f32,
) {
    // A bundle's two records name one behavior, so it drops once.
    let mut dropped = None;
    for running in [brain.motion, brain.action].into_iter().flatten() {
        if dropped != Some(running.behavior)
            && running.should_drop_committed_behavior(&observed_context, now)
        {
            brain.drop_committed_behavior(running, capabilities, now);
            dropped = Some(running.behavior);
        }
    }
    let (context, availability) =
        brain.build_scoring_inputs(observed_context, world_allows_use, capabilities, now);
    let definition = brain.definition();
    let holders = brain.holders();
    if let Some(score_trace) = score_trace {
        score_trace.behaviors.clear();
        score_trace
            .behaviors
            .extend((0..definition.behaviors.len()).map(|index| {
                explain_behavior(definition, index, &context, &availability, holders)
            }));
    }
    let mut missed_start_cooldowns = MotionActionTable::<Option<f32>>::default();
    let assignment =
        assign_behaviors_to_slots(definition, &context, &availability, holders, |index| {
            passes_start_roll(
                rng,
                capabilities,
                definition.behaviors[index],
                now,
                &mut missed_start_cooldowns,
            )
        });
    brain.commit_decision(assignment, &missed_start_cooldowns, capabilities, now);
}

fn passes_start_roll(
    rng: &mut BrainRng,
    capabilities: &Capabilities,
    behavior: Behavior,
    now: f32,
    missed_start_cooldowns: &mut MotionActionTable<Option<f32>>,
) -> bool {
    let Some(motion) = behavior.motion.filter(|motion| motion.is_finite()) else {
        return true;
    };
    let Some(tuning) = capabilities[motion] else {
        return false;
    };
    if tuning.chance >= 1.0 || rng.range(0.0, 1.0) < tuning.chance {
        return true;
    }
    missed_start_cooldowns[motion] = Some(now + tuning.cooldown);
    false
}
