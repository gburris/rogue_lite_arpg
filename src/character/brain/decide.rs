use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use super::considerations::{Behavior, BehaviorScoreExplanation};
#[cfg(feature = "dev_tools")]
use super::diagnostics::DecisionStats;
use super::engine::{assign_behaviors_to_slots, explain_behavior, has_absent_input};
use super::inputs::{ConsiderationInput, DecisionInputComponents, DecisionInputs, DecisionQueries};
use super::{
    Action, Brain, MotionCommands, MotionFailed, MotionFinished, MotionKey, TargetLocationHint,
};
use crate::prelude::*;

/// Longest a committed behavior may run, in case its finish event never arrives.
const COMMIT_TIMEOUT: f32 = 3.0;

/// A behavior that won its slots, with its motion and action copied from the definition. It holds
/// both while it runs. The finish rules for committed and continuously scored behaviors live here.
#[derive(Clone, Copy, Debug)]
pub(super) struct RunningBehavior {
    pub(super) behavior: usize,
    pub(super) motion: Option<MotionKey>,
    pub(super) action: Option<Action>,
    pub(super) committed: bool,
    pub(super) started_at: f32,
}

impl RunningBehavior {
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

/// Applies a [`MotionFinished`]; see [`finish_and_redecide`]. It runs in the flush that despawns
/// the finished motion, so the behavior that held it is gone before the next
/// [`issue_running_motion`] would start it again.
pub(super) fn on_motion_finished(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(finished.character) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    let motion = finished.key;
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_motion").entered();
    finish_and_redecide(
        &mut brain,
        |running| running.motion == Some(motion),
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
}

/// Applies a [`MotionFailed`], so a committed motion the game refused ends at once instead of at
/// its timeout; see [`finish_and_redecide`]. A continuously scored motion is retried instead.
pub(super) fn on_motion_failed(
    failed: On<MotionFailed>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(failed.character) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    let motion = failed.key;
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_motion_failed").entered();
    finish_and_redecide(
        &mut brain,
        |running| running.motion == Some(motion),
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
}

/// Applies an [`ActionFinished`]; see [`finish_and_redecide`]. A finished attack is also noted for
/// the next decision.
pub(super) fn on_action_finished(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(finished.holder) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    let action = Action::Use(finished.key);
    if inputs.action_is_attack(action) {
        brain.record_attack_finished();
    }
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_action").entered();
    finish_and_redecide(
        &mut brain,
        |running| running.action == Some(action),
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
}

/// Applies an [`ActionFailed`], so a committed use the game refused ends at once instead of at its
/// timeout; see [`finish_and_redecide`]. A continuously scored use is retried instead.
pub(super) fn on_action_failed(
    failed: On<ActionFailed>,
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    mut brains: DecidingBrains,
    #[cfg(feature = "dev_tools")] mut decision_stats: ResMut<DecisionStats>,
) {
    let Ok((mut brain, components, mut score_trace)) = brains.get_mut(failed.holder) else {
        return;
    };
    let inputs = DecisionInputs {
        components: &components,
        queries: &queries,
        now: time.elapsed_secs(),
    };
    let action = Action::Use(failed.key);
    #[cfg(feature = "tracy")]
    let _span = info_span!("decide::after_action_failed").entered();
    finish_and_redecide(
        &mut brain,
        |running| running.action == Some(action),
        &inputs,
        score_trace.as_deref_mut(),
        #[cfg(feature = "dev_tools")]
        &mut decision_stats,
    );
}

/// Ends the committed behavior that ran what finished (see
/// [`Brain::finish_committed_behavior`]) and, if one ended, decides again at once.
fn finish_and_redecide(
    brain: &mut Brain,
    ran_what_finished: impl Fn(RunningBehavior) -> bool,
    inputs: &DecisionInputs,
    score_trace: Option<&mut ScoreTrace>,
    #[cfg(feature = "dev_tools")] decision_stats: &mut DecisionStats,
) {
    if brain.finish_committed_behavior(ran_what_finished) {
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

/// Checks in debug builds that each new brain's character can run every behavior the brain lists:
/// it has a source for each motion, and each behavior suits its commit policy (see
/// [`Behavior::suits_commit_policy`]). The policy goes unchecked while a behavior's action source
/// is missing, since items come and go.
pub(super) fn check_new_brains(
    time: Res<Time<Virtual>>,
    queries: DecisionQueries,
    brains: Query<(Entity, &Brain, DecisionInputComponents), Added<Brain>>,
) {
    for (character, brain, components) in &brains {
        let inputs = DecisionInputs {
            components: &components,
            queries: &queries,
            now: time.elapsed_secs(),
        };
        for (index, behavior) in brain.definition().behaviors.iter().enumerate() {
            if let Some(key) = behavior.motion {
                debug_assert!(
                    inputs.has_motion_source(key),
                    "brain behavior {index} of {character:?} would never run: the character has \
                     no {key:?} motion source"
                );
            }
            let has_action_source = behavior
                .action
                .is_none_or(|action| action == Action::Idle || inputs.has_action_source(action));
            debug_assert!(
                !has_action_source || behavior.suits_commit_policy(&inputs),
                "brain behavior {index} of {character:?} would never run: {:?} doesn't suit its \
                 motion {:?} (ends on its own: {}) and action {:?} (ends on its own: {})",
                behavior.commit,
                behavior.motion,
                behavior
                    .motion
                    .is_some_and(|key| inputs.motion_ends_on_its_own(key)),
                behavior.action,
                behavior
                    .action
                    .is_some_and(|action| inputs.action_ends_on_its_own(action)),
            );
        }
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

/// Keeps each character's running motion on the motion its brain holds, steering by the target's
/// [`LocationHint`](super::perception::LocationHint), and stops it when the motion slot is
/// empty; see [`MotionCommands::start`]. A finished motion is not started again, and a refused
/// committed one ends; see [`on_motion_finished`] and [`on_motion_failed`].
pub(super) fn issue_running_motion(
    mut motions: MotionCommands,
    brains: Query<(Entity, &Brain, Option<&TargetLocationHint>)>,
) {
    for (entity, brain, location_hint) in &brains {
        let Some(key) = brain.motion() else {
            motions.stop(entity);
            continue;
        };
        let target = location_hint.map(TargetLocationHint::location_hint);
        motions.start(entity, key, target);
    }
}

/// Keeps each character's uses on the action its brain holds, and stops the others; see
/// [`ActionCommands::start`] and [`ActionCommands::stop`]. Considerations already judged the
/// target, so the game decides only whether a use can start. A use of the held action already
/// running is adopted, so its finish ends a committed behavior, and a finished one is not started
/// again; see [`on_action_finished`].
pub(super) fn issue_running_action(mut actions: ActionCommands, brains: Query<(Entity, &Brain)>) {
    for (entity, brain) in &brains {
        let held_key = brain.action().and_then(Action::key);
        for key in ActionKey::all() {
            if held_key != Some(key) {
                actions.stop(entity, key);
            }
        }
        if let Some(key) = held_key {
            actions.start(entity, key);
        }
    }
}
