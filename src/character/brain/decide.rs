//! The decision pipeline: drop a stale committed behavior, score with inputs read on demand, and
//! commit the winners as [`RunningBehavior`]s. This is the only code that changes which behaviors
//! a [`Brain`] holds.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

#[cfg(feature = "dev_tools")]
use super::diagnostics::DecisionStats;
use super::engine::{assign_behaviors_to_slots, explain_behavior, has_absent_input};
use super::inputs::{ConsiderationInput, DecisionInputComponents, DecisionInputs, DecisionQueries};
use super::issued_motion::IssuedMotion;
use super::vocabulary::{Behavior, BehaviorScoreExplanation};
use super::{Action, ActionOutcome, BehaviorActionFinished, Brain, Motion, MotionFinished};

/// Longest a committed behavior may run, in case its finish event never arrives.
const COMMIT_TIMEOUT: f32 = 3.0;

/// A behavior that won its slots, with its motion and action copied from the definition. Its run
/// id makes it start its motion and action once and lets their finish events name it. The finish
/// rules for committed and continuously scored behaviors live here.
#[derive(Clone, Copy, Debug)]
pub(super) struct RunningBehavior {
    pub(super) behavior: usize,
    pub(super) motion: Option<Motion>,
    pub(super) action: Option<Action>,
    pub(super) behavior_run_id: u32,
    pub(super) committed: bool,
    pub(super) started_at: f32,
}

impl RunningBehavior {
    /// Ends a committed use the game could not start; a continuously scored use waits.
    pub(super) fn report_not_started(self, commands: &mut Commands, entity: Entity) {
        if self.committed
            && let Some(action) = self.action
        {
            commands.trigger(BehaviorActionFinished {
                entity,
                action,
                behavior_run_id: self.behavior_run_id,
                outcome: ActionOutcome::NotStarted,
            });
        }
    }

    /// Whether a committed behavior lost an input it was chosen on or timed out. The game ends
    /// interrupted uses and finite motions itself.
    pub(super) fn should_drop_committed_behavior(
        &self,
        behavior: &Behavior,
        read: &impl Fn(ConsiderationInput) -> Option<f32>,
        inputs: &DecisionInputs,
    ) -> bool {
        self.committed
            && (has_absent_input(behavior, read) || inputs.now - self.started_at > COMMIT_TIMEOUT)
    }
}

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
    pub decided_at: f32,
}

/// Staggers a new brain's first decision; see [`Brain::randomize_decision_phase`].
pub(super) fn schedule_first_decision(mut world: DeferredWorld, context: HookContext) {
    let now = world
        .get_resource::<Time<Virtual>>()
        .map_or(0.0, Time::elapsed_secs);
    if let Some(mut brain) = world.get_mut::<Brain>(context.entity) {
        brain.randomize_decision_phase(now);
    }
}

/// Ends the behavior a finished motion was started for and, if that behavior was still running,
/// decides again immediately.
pub(super) fn on_motion_finished(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    issued_motions: Query<&IssuedMotion>,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(finished.entity) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    let Some(behavior_run_id) = issued_motions
        .get(finished.entity)
        .ok()
        .and_then(|issued_motion| issued_motion.behavior_run_id_of(finished.motion))
    else {
        return;
    };
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_motion").entered();
    finish_and_redecide(
        &mut brain,
        behavior_run_id,
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
}

/// Applies a [`BehaviorActionFinished`], current or stale.
pub(super) fn on_behavior_action_finished(
    finished: On<BehaviorActionFinished>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(finished.entity) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    if finished.outcome == ActionOutcome::Completed && inputs.action_is_attack(finished.action) {
        brain.record_attack_finished();
    }
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_action").entered();
    finish_and_redecide(
        &mut brain,
        finished.behavior_run_id,
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
}

/// Ends the committed behavior stamped `behavior_run_id` and, unless the finish was stale,
/// decides again at once.
fn finish_and_redecide(
    brain: &mut Brain,
    behavior_run_id: u32,
    inputs: &DecisionInputs,
    score_trace: Option<&mut ScoreTrace>,
    #[cfg(feature = "dev_tools")] decision_stats: &mut DecisionStats,
) {
    if brain.finish_committed_behavior(behavior_run_id) {
        redecide_now(brain, inputs, score_trace);
        #[cfg(feature = "dev_tools")]
        decision_stats.record_redecision();
    }
}

/// Decides for every brain whose scheduled decision is due; see
/// [`Brain::advance_decision_clock`].
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let now = time.elapsed_secs();
    for (mut brain, components, mut score_trace) in &mut brains {
        if !brain.advance_decision_clock(now) {
            continue;
        }
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::paced").entered();
        redecide_now(
            &mut brain,
            &DecisionInputs {
                components: &components,
                queries: &queries,
                now,
            },
            score_trace.as_deref_mut(),
        );
        #[cfg(feature = "dev_tools")]
        decision_stats.record_paced_decision();
    }
}

/// Drops stale committed behaviors, then fills the slots. A winner with a start chance must pass
/// its roll; see [`Behavior::with_start_chance`].
fn redecide_now(brain: &mut Brain, inputs: &DecisionInputs, score_trace: Option<&mut ScoreTrace>) {
    brain.drop_stale_committed_behaviors(inputs);
    let definition = brain.definition();
    let running_behavior_ids = brain.running_behavior_ids();
    let read = |input: ConsiderationInput| inputs.read_consideration_input(input, brain);
    let available = |index: usize| brain.is_available(index, inputs);
    if let Some(score_trace) = score_trace {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::explain").entered();
        score_trace.decided_at = inputs.now;
        score_trace.behavior_explanations.clear();
        score_trace
            .behavior_explanations
            .extend((0..definition.behaviors.len()).map(|index| {
                explain_behavior(definition, index, &read, &available, &running_behavior_ids)
            }));
    }
    let mut missed_start_ids = Vec::new();
    let winning_behavior_ids = {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::assign").entered();
        assign_behaviors_to_slots(
            definition,
            &read,
            &available,
            &running_behavior_ids,
            |index| passes_start_roll(definition.behaviors[index], index, &mut missed_start_ids),
        )
    };
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::commit").entered();
    brain.commit_decision(&winning_behavior_ids, &missed_start_ids, inputs);
}

fn passes_start_roll(behavior: Behavior, index: usize, missed_start_ids: &mut Vec<usize>) -> bool {
    let Some(start_chance) = behavior.start_chance else {
        return true;
    };
    if start_chance.chance >= 1.0 || rand::rng().random::<f32>() < start_chance.chance {
        return true;
    }
    missed_start_ids.push(index);
    false
}
