//! The decision tick: scores the staged perception and writes intents. This is
//! the only code that changes a [`Brain`]'s motion.

use bevy::prelude::*;

use super::scoring::{choose_actions, choose_movement};
use super::motion::{MotionAborted, MotionFinished};
use super::senses::{Perception, motion_readiness};
use super::{
    Action, ActionFinished, ActionIntent, Brain, BrainRng, Cooldown, DecisionContext, Motion,
};

/// How long a swoop or dash may go without line of sight before it is
/// dropped, so one that clips a pillar carries on.
const LOS_GRACE_PERIOD: f32 = 1.0;
/// Longest a locked motion may run, a safety net against a missing finish.
const LOCK_TIMEOUT: f32 = 3.0;

/// Releases a finite motion its executor reported finished, starts its
/// cooldown, and decides again for that entity at once instead of waiting for
/// the next tick. The perception is the one staged at the last tick, so the
/// motion state it carries is refreshed first.
pub(super) fn release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &mut Perception, Mut<ActionIntent>)>,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, mut perception, mut action_intent)) =
        brains.get_mut(finished.entity)
    else {
        return;
    };
    let cooldown =
        perception.motion_tuning[finished.motion.index()].map_or(0.0, |tuning| tuning.cooldown);
    if !brain.finish_motion(finished.motion, now + cooldown) {
        return;
    }
    redecide(&mut brain, &mut rng, &mut perception, &mut action_intent, now);
}

/// Drops a finite motion its executor could not start, with no cooldown, and
/// decides again for that entity at once.
pub(super) fn abort_failed_motion(
    aborted: On<MotionAborted>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &mut Perception, Mut<ActionIntent>)>,
) {
    let Ok((mut brain, mut rng, mut perception, mut action_intent)) =
        brains.get_mut(aborted.entity)
    else {
        return;
    };
    if !brain
        .active_motion()
        .is_some_and(|active| active.motion == aborted.motion)
    {
        return;
    }
    brain.cancel_motion();
    redecide(
        &mut brain,
        &mut rng,
        &mut perception,
        &mut action_intent,
        time.elapsed_secs(),
    );
}

/// Decides between ticks. The staged perception still describes the motion
/// that just ended, so its motion state is refreshed first.
fn redecide(
    brain: &mut Brain,
    rng: &mut BrainRng,
    perception: &mut Perception,
    action_intent: &mut Mut<ActionIntent>,
    now: f32,
) {
    perception.motion = brain.motion();
    perception.motion_ready = motion_readiness(brain, &perception.motion_tuning, now);
    decide(brain, rng, perception, action_intent, now);
}

/// Starts an ability's cooldown when its use finishes.
pub(super) fn start_ability_cooldown(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &Perception)>,
) {
    let Some(action @ Action::Ability(ability)) = finished.action else {
        return;
    };
    let Ok((mut brain, perception)) = brains.get_mut(finished.entity) else {
        return;
    };
    let cooldown = perception.action_tuning[action.index()].map_or(0.0, |tuning| tuning.cooldown);
    brain.start_cooldown(Cooldown::Ability(ability), time.elapsed_secs() + cooldown);
}

/// Scores the staged perception and writes the outcome back: the movement
/// layer's motion, and the action intent.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &Perception, Mut<ActionIntent>)>,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, perception, mut action_intent) in &mut brains {
        decide(&mut brain, &mut rng, perception, &mut action_intent, now);
    }
}

fn decide(
    brain: &mut Brain,
    rng: &mut BrainRng,
    perception: &Perception,
    action_intent: &mut Mut<ActionIntent>,
    now: f32,
) {
    let definition = brain.definition;
    let mut context = perception.context;
    if lock_broken(brain, &context, now) {
        brain.cancel_motion();
        context.motion = brain.motion();
    }
    brain.continuous_motion = choose_motion(brain, rng, &context, now);
    let actions = choose_actions(definition, &context);
    if action_intent.0 != actions {
        action_intent.0 = actions;
    }
}

/// Whether the running finite motion must give up: the target is gone, the
/// body is stunned, line of sight has been lost past the grace period, or the
/// lock has outlived its safety timeout. An abort starts no cooldown.
fn lock_broken(brain: &mut Brain, context: &DecisionContext, now: f32) -> bool {
    let Some(active) = brain.active.as_mut() else {
        return false;
    };
    let Some(target) = context.target else {
        return true;
    };
    if context.stunned || now - active.started_at > LOCK_TIMEOUT {
        return true;
    }
    if target.line_of_sight {
        active.los_lost_since = None;
        return false;
    }
    let lost_since = *active.los_lost_since.get_or_insert(now);
    now - lost_since > LOS_GRACE_PERIOD
}

/// Runs the movement layer and returns the continuous motion to drive. A running finite
/// motion is committed: it is never re-scored and ends when its executor
/// reports it or [`lock_broken`] says so. A winning finite motion rolls
/// its start chance; a miss waits out its cooldown and the layer picks again
/// without it.
fn choose_motion(
    brain: &mut Brain,
    rng: &mut BrainRng,
    context: &DecisionContext,
    now: f32,
) -> Motion {
    if brain.active.is_some() {
        return brain.continuous_motion;
    }
    let mut declined = [false; Motion::COUNT];
    loop {
        let can_start = |motion: Motion| context.can_start(motion) && !declined[motion.index()];
        match choose_movement(
            brain.definition,
            context,
            brain.continuous_motion,
            can_start,
        ) {
            None => return brain.continuous_motion,
            Some(motion) if !motion.is_finite() => return motion,
            Some(motion) => {
                let Some(tuning) = context.motion_tuning[motion.index()] else {
                    declined[motion.index()] = true;
                    continue;
                };
                if tuning.chance >= 1.0 || rng.range(0.0, 1.0) < tuning.chance {
                    brain.start_motion(motion, now);
                    return brain.continuous_motion;
                }
                brain.start_cooldown(Cooldown::Motion(motion), now + tuning.cooldown);
                declined[motion.index()] = true;
            }
        }
    }
}
