//! The decision tick: scores the staged perception and fills the motion and
//! action slots. This is the only code that changes which behaviors a
//! [`Brain`] holds.

use bevy::prelude::*;

use super::model::Behavior;
use super::motion::{MotionAborted, MotionFinished};
use super::scoring::assign_behaviors_to_slots;
use super::senses::{Perception, finite_motion_readiness};
use super::{Action, ActionAborted, ActionFinished, Brain, BrainRng, Cooldown, DecisionContext};

/// How long a swoop or dash may go without line of sight before it is
/// dropped, so one that clips a pillar carries on.
const LOS_GRACE_PERIOD: f32 = 1.0;
/// Longest a locked motion may run, a safety net against a missing finish.
const LOCK_TIMEOUT: f32 = 3.0;

/// Marks a locked motion's part finished and starts its cooldown. Once every
/// finite part of the lock is done it releases, and the entity decides again
/// at once instead of waiting for the next tick. The perception is the one staged at the last tick, so the
/// motion state it carries is refreshed first.
pub(super) fn release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &mut Perception)>,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, mut perception)) = brains.get_mut(finished.entity) else {
        return;
    };
    let cooldown =
        perception.motion_tuning[finished.motion.index()].map_or(0.0, |tuning| tuning.cooldown);
    let Some(released) = brain.finish_motion_part(finished.motion, now) else {
        return;
    };
    brain.start_cooldown(Cooldown::Motion(finished.motion), now + cooldown);
    if released {
        decide_again_immediately(&mut brain, &mut rng, &mut perception, now);
    }
}

/// Drops a finite motion its executor could not start, with no cooldown, and
/// decides again for that entity at once.
pub(super) fn abort_failed_motion(
    aborted: On<MotionAborted>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &mut Perception)>,
) {
    let Ok((mut brain, mut rng, mut perception)) = brains.get_mut(aborted.entity) else {
        return;
    };
    if !brain
        .active_motion()
        .is_some_and(|active| active.motion == aborted.motion)
    {
        return;
    }
    brain.abort_locked_behavior();
    decide_again_immediately(&mut brain, &mut rng, &mut perception, time.elapsed_secs());
}

/// Drops a locked action its applier could not start, with no cooldown, and
/// decides again for that entity at once.
pub(super) fn abort_failed_action(
    aborted: On<ActionAborted>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &mut Perception)>,
) {
    let Ok((mut brain, mut rng, mut perception)) = brains.get_mut(aborted.entity) else {
        return;
    };
    if brain.active_action().is_none() {
        return;
    }
    brain.abort_locked_behavior();
    decide_again_immediately(&mut brain, &mut rng, &mut perception, time.elapsed_secs());
}

/// Decides between ticks. The staged perception still describes the motion
/// that just ended, so its motion state is refreshed first.
fn decide_again_immediately(
    brain: &mut Brain,
    rng: &mut BrainRng,
    perception: &mut Perception,
    now: f32,
) {
    perception.motion = brain.motion();
    perception.motion_ready = finite_motion_readiness(brain, &perception.motion_tuning, now);
    decide_slot_assignments(brain, rng, perception, now);
}

/// Starts an ability's cooldown when its use finishes, and marks a locked
/// action's part finished. Once every finite part of the lock is done it
/// releases and the entity decides again at once. Equipment finishing is the
/// attack-finished signal the senses would otherwise only pick up next tick.
pub(super) fn finish_action(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &mut Perception)>,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, mut perception)) = brains.get_mut(finished.entity) else {
        return;
    };
    if let Some(action @ Action::Ability(ability)) = finished.action {
        let cooldown =
            perception.action_tuning[action.index()].map_or(0.0, |tuning| tuning.cooldown);
        brain.start_cooldown(Cooldown::Ability(ability), now + cooldown);
    }
    if finished.action.is_none() {
        perception.attack_finished = true;
    }
    if brain.finish_action_part(finished.action, now) == Some(true) {
        decide_again_immediately(&mut brain, &mut rng, &mut perception, now);
    }
}

/// Scores the staged perception and writes the outcome back: which behaviors
/// hold the motion and action slots.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &Perception)>,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, perception) in &mut brains {
        decide_slot_assignments(&mut brain, &mut rng, perception, now);
    }
}

/// Fills the free slots. A locked behavior is never re-scored: it ends when
/// its executor reports it or [`should_abort_lock`] says so. A winning locked motion
/// rolls its start chance; a miss waits out its cooldown and the next best
/// behavior takes the slot. A slot nothing claims keeps its holder.
fn decide_slot_assignments(
    brain: &mut Brain,
    rng: &mut BrainRng,
    perception: &Perception,
    now: f32,
) {
    let mut context = perception.context;
    if brain.release_if_complete(now) {
        context.motion = brain.motion();
    }
    if should_abort_lock(brain, &context, now) {
        brain.abort_locked_behavior();
        context.motion = brain.motion();
    }
    let definition = brain.definition();
    let assignment = assign_behaviors_to_slots(definition, &context, brain.holders(), |index| {
        passes_start_roll(brain, rng, &context, definition.behaviors[index].value, now)
    });
    brain.assign_slots(assignment, now);
}

/// Whether a winning behavior actually starts: a finite motion rolls its
/// chance, and a miss starts its cooldown.
fn passes_start_roll(
    brain: &mut Brain,
    rng: &mut BrainRng,
    context: &DecisionContext,
    behavior: Behavior,
    now: f32,
) -> bool {
    let Some(motion) = behavior.motion.filter(|motion| motion.is_finite()) else {
        return true;
    };
    let Some(tuning) = context.motion_tuning[motion.index()] else {
        return false;
    };
    if tuning.chance >= 1.0 || rng.range(0.0, 1.0) < tuning.chance {
        return true;
    }
    brain.start_cooldown(Cooldown::Motion(motion), now + tuning.cooldown);
    false
}

/// Whether the lock must give up: the target is gone, the body is
/// stunned, line of sight has been lost past the grace period, or the lock has
/// outlived its safety timeout. An abort starts no cooldown.
fn should_abort_lock(brain: &mut Brain, context: &DecisionContext, now: f32) -> bool {
    let Some(running) = brain.locked_running_behavior_mut() else {
        return false;
    };
    let Some(target) = context.target else {
        return true;
    };
    if context.stunned || now - running.started_at > LOCK_TIMEOUT {
        return true;
    }
    if target.line_of_sight {
        running.los_lost_since = None;
        return false;
    }
    let lost_since = *running.los_lost_since.get_or_insert(now);
    now - lost_since > LOS_GRACE_PERIOD
}
