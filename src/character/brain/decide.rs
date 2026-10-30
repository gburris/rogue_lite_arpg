//! The decision pipeline: build the decision context, drop a stale committed behavior,
//! score, and commit. This is the only code that changes which behaviors a
//! [`Brain`] holds.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use super::capability::{
    CapabilityComponents, CapabilityProfiles, MotionActionTable, build_capability_profiles,
};
use super::engine::{
    Behavior, BehaviorScoreExplanation, assign_behaviors_to_slots, explain_behavior,
};
use super::motion::MotionFinished;
use super::perception::{
    DecisionContext, DecisionInputComponents, DecisionInputComponentsItem, UseEligibilityQueries,
    build_decision_context,
};
use super::{Action, ActionFinished, ActionOutcome, Brain};

/// Seconds between a brain's scheduled decisions.
const DECISION_PERIOD: f32 = 0.1;

type DecidingBrains<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut Brain,
        DecisionInputComponents,
        CapabilityComponents,
        Option<&'static mut ScoreTrace>,
    ),
>;

/// Debug readout of a brain's last decision: one explanation per behavior, in
/// definition order. Only brains carrying it pay for the explanations.
#[derive(Component, Debug, Default)]
pub struct ScoreTrace {
    pub behavior_explanations: Vec<BehaviorScoreExplanation>,
}

/// Puts a new brain's first decision at a random point in the next period, so brains spawned
/// together don't all decide on the same frame.
pub(super) fn schedule_first_decision(mut world: DeferredWorld, context: HookContext) {
    let now = world
        .get_resource::<Time<Virtual>>()
        .map_or(0.0, Time::elapsed_secs);
    if let Some(mut brain) = world.get_mut::<Brain>(context.entity) {
        brain.next_decision_at = now + rand::rng().random_range(0.0..DECISION_PERIOD);
    }
}

pub(super) fn on_release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    use_eligibility_queries: UseEligibilityQueries,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, decision_input_components, components, mut score_trace)) =
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
pub(super) fn on_finish_action(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    use_eligibility_queries: UseEligibilityQueries,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, decision_input_components, components, mut score_trace)) =
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
    for (mut brain, decision_input_components, components, mut score_trace) in &mut brains {
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
    context_without_memory: DecisionContext,
    capability_profiles: &CapabilityProfiles,
    score_trace: Option<&mut ScoreTrace>,
    now: f32,
) {
    for running in brain.running_behaviors.clone() {
        if running.should_drop_committed_behavior(&context_without_memory, now) {
            brain.drop_committed_behavior(running, capability_profiles, now);
        }
    }
    let (context, available_motion_action_table) =
        brain.build_scoring_inputs(context_without_memory, capability_profiles, now);
    let definition = brain.definition();
    let running_behavior_ids = brain.running_behavior_ids();
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
                    &running_behavior_ids,
                )
            }));
    }
    let mut missed_start_cooldowns = MotionActionTable::<Option<f32>>::default();
    let winning_behavior_ids = assign_behaviors_to_slots(
        definition,
        &context,
        &available_motion_action_table,
        &running_behavior_ids,
        |index| {
            passes_start_roll(
                capability_profiles,
                definition.behaviors[index],
                now,
                &mut missed_start_cooldowns,
            )
        },
    );
    brain.commit_decision(
        &winning_behavior_ids,
        &missed_start_cooldowns,
        capability_profiles,
        now,
    );
}

fn passes_start_roll(
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
    if capability_profile.chance >= 1.0 || rand::rng().random::<f32>() < capability_profile.chance {
        return true;
    }
    missed_start_cooldowns[motion] = Some(now + capability_profile.cooldown);
    false
}
