//! The decision pipeline: build the decision context, drop a stale committed behavior,
//! score, and commit. This is the only code that changes which behaviors a
//! [`Brain`] holds.

use bevy::prelude::*;

use super::capability::{
    CapabilityComponents, CapabilityProfiles, MotionActionTable, build_capability_profiles,
};
use super::engine::{Behavior, assign_behaviors_to_slots, explain_behavior};
use super::motion::MotionFinished;
use super::perception::{
    DecisionContext, DecisionInputComponents, DecisionInputComponentsItem, UseEligibilityQueries,
    build_decision_context,
};
use super::{Action, ActionFinished, ActionOutcome, Brain, BrainRng, DECISION_PERIOD, ScoreTrace};

type DecidingBrains<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut Brain,
        &'static mut BrainRng,
        DecisionInputComponents,
        CapabilityComponents,
        Option<&'static mut ScoreTrace>,
    ),
>;

pub(super) fn release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    use_eligibility_queries: UseEligibilityQueries,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, decision_input_components, components, mut score_trace)) =
        brains.get_mut(finished.entity)
    else {
        return;
    };
    let capability_profiles =
        build_capability_profiles(&components, &use_eligibility_queries.equipment);
    let cooldown = capability_profiles[finished.motion].map_or(0.0, |profile| profile.cooldown);
    brain.start_cooldown(finished.motion, now + cooldown);
    if brain.finish_committed_behavior(finished.behavior_run_id) {
        redecide_now(
            &mut brain,
            &mut rng,
            &decision_input_components,
            &capability_profiles,
            &use_eligibility_queries,
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
    use_eligibility_queries: UseEligibilityQueries,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, decision_input_components, components, mut score_trace)) =
        brains.get_mut(finished.entity)
    else {
        return;
    };
    let capability_profiles =
        build_capability_profiles(&components, &use_eligibility_queries.equipment);
    match finished.action {
        Action::Use(slot)
            if slot.is_instant_use() && finished.outcome == ActionOutcome::Completed =>
        {
            brain.attack_finish_pending = true;
        }
        Action::Ability(ability) => {
            let cooldown = capability_profiles[ability].map_or(0.0, |profile| profile.cooldown);
            brain.start_cooldown(ability, now + cooldown);
        }
        // A held or unstarted use is no attack, and nothing reports an idle
        // action's finish.
        Action::Use(_) | Action::Idle => {}
    }
    if brain.finish_committed_behavior(finished.behavior_run_id) {
        redecide_now(
            &mut brain,
            &mut rng,
            &decision_input_components,
            &capability_profiles,
            &use_eligibility_queries,
            score_trace.as_deref_mut(),
            now,
        );
    }
}

/// Decides for every due brain. Each moves to its next period boundary after
/// now, keeping its phase without a burst of catch-up decisions after a long frame.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    use_eligibility_queries: UseEligibilityQueries,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, decision_input_components, components, mut score_trace) in &mut brains
    {
        if now < brain.next_decision_at {
            continue;
        }
        let periods_passed = ((now - brain.next_decision_at) / DECISION_PERIOD).floor() + 1.0;
        brain.next_decision_at += periods_passed * DECISION_PERIOD;
        // Rounding near an exact multiple can land the boundary on `now`.
        if brain.next_decision_at <= now {
            brain.next_decision_at += DECISION_PERIOD;
        }
        let capability_profiles =
            build_capability_profiles(&components, &use_eligibility_queries.equipment);
        redecide_now(
            &mut brain,
            &mut rng,
            &decision_input_components,
            &capability_profiles,
            &use_eligibility_queries,
            score_trace.as_deref_mut(),
            now,
        );
    }
}

fn redecide_now(
    brain: &mut Brain,
    rng: &mut BrainRng,
    decision_input_components: &DecisionInputComponentsItem,
    capability_profiles: &CapabilityProfiles,
    use_eligibility_queries: &UseEligibilityQueries,
    score_trace: Option<&mut ScoreTrace>,
    now: f32,
) {
    let context_without_memory = build_decision_context(
        decision_input_components,
        capability_profiles,
        use_eligibility_queries,
        now,
    );
    decide_slot_assignments(
        brain,
        rng,
        context_without_memory,
        capability_profiles,
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
    context_without_memory: DecisionContext,
    capability_profiles: &CapabilityProfiles,
    score_trace: Option<&mut ScoreTrace>,
    now: f32,
) {
    // A bundle's two records name one behavior, so it drops once.
    let mut dropped = None;
    for running in [brain.motion_slot, brain.action_slot].into_iter().flatten() {
        if dropped != Some(running.behavior)
            && running.should_drop_committed_behavior(&context_without_memory, now)
        {
            brain.drop_committed_behavior(running, capability_profiles, now);
            dropped = Some(running.behavior);
        }
    }
    let (context, available_motion_action_table) =
        brain.build_scoring_inputs(context_without_memory, capability_profiles, now);
    let definition = brain.definition();
    let holders = brain.slot_holders();
    if let Some(score_trace) = score_trace {
        score_trace.behavior_explanations.clear();
        score_trace
            .behavior_explanations
            .extend((0..definition.behaviors.len()).map(|index| {
                explain_behavior(
                    definition,
                    index,
                    &context,
                    &available_motion_action_table,
                    holders,
                )
            }));
    }
    let mut missed_start_cooldowns = MotionActionTable::<Option<f32>>::default();
    let assignment = assign_behaviors_to_slots(
        definition,
        &context,
        &available_motion_action_table,
        holders,
        |index| {
            passes_start_roll(
                rng,
                capability_profiles,
                definition.behaviors[index],
                now,
                &mut missed_start_cooldowns,
            )
        },
    );
    brain.commit_decision(
        assignment,
        &missed_start_cooldowns,
        capability_profiles,
        now,
    );
}

fn passes_start_roll(
    rng: &mut BrainRng,
    capability_profiles: &CapabilityProfiles,
    behavior: Behavior,
    now: f32,
    missed_start_cooldowns: &mut MotionActionTable<Option<f32>>,
) -> bool {
    let Some(motion) = behavior.motion.filter(|motion| motion.is_finite()) else {
        return true;
    };
    let Some(capability_profile) = capability_profiles[motion] else {
        return false;
    };
    if capability_profile.chance >= 1.0 || rng.range(0.0, 1.0) < capability_profile.chance {
        return true;
    }
    missed_start_cooldowns[motion] = Some(now + capability_profile.cooldown);
    false
}
