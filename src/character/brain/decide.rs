//! The decision tick: scores the staged perception and writes intents. This is
//! the only code that changes a [`Brain`]'s motion.

use bevy::prelude::*;

use super::scoring::{choose_actions, choose_movement};
use super::motion::MotionFinished;
use super::senses::{Perception, motion_readiness};
use super::{ActionIntent, Brain, BrainRng, DecisionContext, Motion};

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
    perception.motion = brain.motion();
    perception.motion_ready = motion_readiness(&brain, &perception.motion_tuning, now);
    decide(&mut brain, &mut rng, &perception, &mut action_intent, now);
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
    brain.continuous_motion = choose_motion(brain, rng, &perception.context, now);
    let actions = choose_actions(definition, &perception.context);
    if action_intent.0 != actions {
        action_intent.0 = actions;
    }
}

/// Runs the movement layer and returns the continuous motion to drive. A running finite
/// motion is committed: it is never re-scored and ends when its executor
/// reports it, or here when the target is gone. A winning finite motion rolls
/// its start chance; a miss waits out its cooldown and the layer picks again
/// without it.
fn choose_motion(
    brain: &mut Brain,
    rng: &mut BrainRng,
    context: &DecisionContext,
    now: f32,
) -> Motion {
    if brain.active.is_some() {
        if context.target.is_some() {
            return brain.continuous_motion;
        }
        brain.cancel_motion();
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
                    brain.start_motion(motion);
                    return brain.continuous_motion;
                }
                brain.defer_motion(motion, now + tuning.cooldown);
                declined[motion.index()] = true;
            }
        }
    }
}
