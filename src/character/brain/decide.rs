//! The decision tick: scores the staged perception and writes intents. This is
//! the only code that changes a [`Brain`]'s motion.

use bevy::prelude::*;

use super::scoring::{choose_actions, choose_movement};
use super::senses::Perception;
use super::{ActionIntent, Brain, BrainRng, DecisionContext, Motion, MotionFinished};

/// Releases finite motions their executors reported finished and starts their
/// cooldowns. Runs every frame so a finished motion never lingers a tick.
pub(super) fn end_finished_motions(
    time: Res<Time<Virtual>>,
    mut finished: MessageReader<MotionFinished>,
    mut brains: Query<(&mut Brain, &Perception)>,
) {
    let now = time.elapsed_secs();
    for message in finished.read() {
        let Ok((mut brain, perception)) = brains.get_mut(message.entity) else {
            continue;
        };
        let cooldown =
            perception.motion_tuning[message.motion.index()].map_or(0.0, |tuning| tuning.cooldown);
        brain.finish_motion(message.motion, now + cooldown);
    }
}

/// Scores the staged perception and writes the outcome back: the movement
/// layer's motion, and the action intent.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &Perception, &mut ActionIntent)>,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, perception, mut action_intent) in &mut brains {
        let definition = brain.definition;
        brain.continuous_motion = choose_motion(&mut brain, &mut rng, &perception.context, now);
        let actions = choose_actions(definition, &perception.context);
        if action_intent.0 != actions {
            action_intent.0 = actions;
        }
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
