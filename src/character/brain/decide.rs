//! The decision pipeline: drop a stale committed behavior, score with inputs read on
//! demand, and commit. This is the only code that changes which behaviors a
//! [`Brain`] holds.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use super::capability::MotionActionTable;
#[cfg(feature = "dev_tools")]
use super::diagnostics::DecisionStats;
use super::engine::{
    Behavior, BehaviorScoreExplanation, assign_behaviors_to_slots, explain_behavior,
};
use super::inputs::{ConsiderationInput, DecisionInputComponents, DecisionInputs, DecisionQueries};
use super::motion::MotionFinished;
use super::{Action, ActionFinished, ActionOutcome, Brain};

/// Seconds between a brain's scheduled decisions.
const DECISION_PERIOD: f32 = 0.1;

type DecidingBrains<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut Brain,
        DecisionInputComponents,
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

/// Starts a finished motion's cooldown and, if its run is still current, decides again at once.
pub(super) fn on_release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, item, mut score_trace)) = brains.get_mut(finished.entity) else {
        return;
    };
    let inputs = DecisionInputs {
        item: &item,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    let cooldown = inputs
        .motion_profile(finished.motion)
        .map_or(0.0, |profile| profile.cooldown);
    brain.start_cooldown(finished.motion, inputs.now + cooldown);
    if brain.finish_committed_behavior(finished.behavior_run_id) {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::after_motion").entered();
        redecide_now(&mut brain, &inputs, score_trace.as_deref_mut());
        #[cfg(feature = "dev_tools")]
        decision_stats.record_redecision();
    }
}

/// Applies an [`ActionFinished`], current or stale. An ability starts its
/// cooldown even if it never started, so the next decision can't pick it
/// straight back.
pub(super) fn on_finish_action(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, item, mut score_trace)) = brains.get_mut(finished.entity) else {
        return;
    };
    let inputs = DecisionInputs {
        item: &item,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    match finished.action {
        Action::Use(slot)
            if slot.is_instant_use() && finished.outcome == ActionOutcome::Completed =>
        {
            brain.attack_finish_pending = true;
        }
        Action::Ability(ability) => {
            let cooldown = inputs
                .action_profile(finished.action)
                .map_or(0.0, |profile| profile.cooldown);
            brain.start_cooldown(ability, inputs.now + cooldown);
        }
        // A held or unstarted use is no attack, and nothing reports an idle
        // action's finish.
        Action::Use(_) | Action::Idle => {}
    }
    if brain.finish_committed_behavior(finished.behavior_run_id) {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::after_action").entered();
        redecide_now(&mut brain, &inputs, score_trace.as_deref_mut());
        #[cfg(feature = "dev_tools")]
        decision_stats.record_redecision();
    }
}

/// Decides for every due brain. Each moves to its next period boundary after
/// now, keeping its phase without a burst of catch-up decisions after a long frame.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let now = time.elapsed_secs();
    for (mut brain, item, mut score_trace) in &mut brains {
        if now < brain.next_decision_at {
            continue;
        }
        let periods_passed = ((now - brain.next_decision_at) / DECISION_PERIOD).floor() + 1.0;
        brain.next_decision_at += periods_passed * DECISION_PERIOD;
        // Rounding near an exact multiple can land the boundary on `now`.
        if brain.next_decision_at <= now {
            brain.next_decision_at += DECISION_PERIOD;
        }
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::paced").entered();
        redecide_now(
            &mut brain,
            &DecisionInputs {
                item: &item,
                queries: &queries,
                now,
            },
            score_trace.as_deref_mut(),
        );
        #[cfg(feature = "dev_tools")]
        decision_stats.record_paced_decision();
    }
}

/// Drops committed behaviors that lost their target, were stunned or timed
/// out, then fills the slots. A winning finite motion rolls its start chance; a
/// miss waits out its cooldown and the next best behavior takes the slot.
fn redecide_now(brain: &mut Brain, inputs: &DecisionInputs, score_trace: Option<&mut ScoreTrace>) {
    let (has_target, stunned) = (inputs.target().is_some(), inputs.stunned());
    for running in brain.running_behaviors.clone() {
        if running.should_drop_committed_behavior(has_target, stunned, inputs.now) {
            brain.drop_committed_behavior(running, inputs);
        }
    }
    // Read after the drops, which change the running motion and cooldowns.
    let memory = brain.memory();
    let definition = brain.definition();
    let running_behavior_ids = brain.running_behavior_ids();
    let read = |input: ConsiderationInput| inputs.read(input, memory);
    let available = |behavior: &Behavior| inputs.is_available(behavior, brain);
    if let Some(score_trace) = score_trace {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::explain").entered();
        score_trace.behavior_explanations.clear();
        score_trace
            .behavior_explanations
            .extend((0..definition.behaviors.len()).map(|index| {
                explain_behavior(definition, index, &read, &available, &running_behavior_ids)
            }));
    }
    let mut missed_start_cooldowns = MotionActionTable::<Option<f32>>::default();
    let winning_behavior_ids = {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::assign").entered();
        assign_behaviors_to_slots(
            definition,
            &read,
            &available,
            &running_behavior_ids,
            |index| {
                passes_start_roll(
                    inputs,
                    definition.behaviors[index],
                    &mut missed_start_cooldowns,
                )
            },
        )
    };
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::commit").entered();
    brain.commit_decision(&winning_behavior_ids, &missed_start_cooldowns, inputs);
}

fn passes_start_roll(
    inputs: &DecisionInputs,
    behavior: Behavior,
    missed_start_cooldowns: &mut MotionActionTable<Option<f32>>,
) -> bool {
    let Some(motion) = behavior.motion.filter(|motion| motion.is_finite()) else {
        return true;
    };
    let Some(capability_profile) = inputs.motion_profile(motion) else {
        return false;
    };
    if capability_profile.chance >= 1.0 || rand::rng().random::<f32>() < capability_profile.chance {
        return true;
    }
    missed_start_cooldowns[motion] = Some(inputs.now + capability_profile.cooldown);
    false
}
