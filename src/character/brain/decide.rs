//! The decision pipeline: drop a stale committed behavior, score with inputs read on demand, and
//! commit the winners as [`RunningBehavior`]s. This is the only code that changes which behaviors
//! a [`Brain`] holds.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

#[cfg(feature = "dev_tools")]
use super::diagnostics::DecisionStats;
use super::engine::{
    Behavior, BehaviorScoreExplanation, assign_behaviors_to_slots, explain_behavior,
};
use super::inputs::{ConsiderationInput, DecisionInputComponents, DecisionInputs, DecisionQueries};
use super::motion::MotionFinished;
use super::{Action, ActionFinished, ActionOutcome, Brain, Motion};

/// Longest a committed behavior may run, a safety net against a missing finish.
const COMMIT_TIMEOUT: f32 = 3.0;

/// A behavior that won its slots, with its motion and action copied from the
/// definition and stamped with a run id so a finite motion's
/// `CurrentBehaviorRunId` can tell a fresh
/// start from the run it just finished. Appliers report through it, so the
/// committed/continuously scored finish policy lives here.
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
    pub(super) fn report_completed(self, commands: &mut Commands, entity: Entity) {
        if let Some(finished) = self.build_action_finished(entity, ActionOutcome::Completed) {
            commands.trigger(finished);
        }
    }

    /// Ends a committed use the live world refused; a continuously scored use waits.
    pub(super) fn report_not_started(self, commands: &mut Commands, entity: Entity) {
        if self.committed
            && let Some(finished) = self.build_action_finished(entity, ActionOutcome::NotStarted)
        {
            commands.trigger(finished);
        }
    }

    /// `None` for a record with no action to report on.
    fn build_action_finished(
        self,
        entity: Entity,
        outcome: ActionOutcome,
    ) -> Option<ActionFinished> {
        Some(ActionFinished {
            entity,
            action: self.action?,
            behavior_run_id: self.behavior_run_id,
            outcome,
        })
    }

    /// Whether a committed behavior lost its target, was stunned, or timed out.
    pub(super) fn should_drop_committed_behavior(&self, inputs: &DecisionInputs) -> bool {
        self.committed
            && (inputs.target().is_none()
                || inputs.is_stunned()
                || inputs.now - self.started_at > COMMIT_TIMEOUT)
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

/// Starts a finished motion's cooldown and, if its run is still current, decides again at once.
pub(super) fn on_release_finished_motion(
    finished: On<MotionFinished>,
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
    brain.start_motion_cooldown(finished.motion, &inputs);
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_motion").entered();
    finish_and_redecide(
        &mut brain,
        finished.behavior_run_id,
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
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
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(finished.entity) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    match finished.action {
        Action::Use(slot)
            if slot.is_instant_use() && finished.outcome == ActionOutcome::Completed =>
        {
            brain.record_attack_finished();
        }
        Action::Ability(ability) => brain.start_ability_cooldown(ability, &inputs),
        // A held or unstarted use is no attack, and nothing reports an idle
        // action's finish.
        Action::Use(_) | Action::Idle => {}
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

/// Drops stale committed behaviors, then fills the slots. A winning finite motion rolls its
/// start chance; a miss waits out its cooldown and the next best behavior takes the slot.
fn redecide_now(brain: &mut Brain, inputs: &DecisionInputs, score_trace: Option<&mut ScoreTrace>) {
    brain.drop_stale_committed_behaviors(inputs);
    let definition = brain.definition();
    let running_behavior_ids = brain.running_behavior_ids();
    let read = |input: ConsiderationInput| inputs.read_consideration_input(input, brain);
    let available = |behavior: &Behavior| brain.is_available(behavior, inputs);
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
    let mut missed_starts = Vec::new();
    let winning_behavior_ids = {
        #[cfg(feature = "tracy")]
        let _span = info_span!("decide::assign").entered();
        assign_behaviors_to_slots(
            definition,
            &read,
            &available,
            &running_behavior_ids,
            |index| passes_start_roll(inputs, definition.behaviors[index], &mut missed_starts),
        )
    };
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::commit").entered();
    brain.commit_decision(&winning_behavior_ids, &missed_starts, inputs);
}

fn passes_start_roll(
    inputs: &DecisionInputs,
    behavior: Behavior,
    missed_starts: &mut Vec<Motion>,
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
    missed_starts.push(motion);
    false
}
