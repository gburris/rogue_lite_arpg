//! The decision tick: scores the staged perception and writes intents. This is
//! the only code that changes a [`Brain`]'s mode or maneuver.

use bevy::prelude::*;

use super::scoring::{choose_actions, choose_movement};
use super::senses::Perception;
use super::{
    ActionIntent, Brain, BrainRng, DecisionContext, Maneuver, ManeuverFinished, MotionMode,
    Movement,
};

/// Releases maneuvers their executors reported finished and starts their
/// cooldowns. Runs every frame so a finished maneuver never lingers a tick.
pub(super) fn end_finished_maneuvers(
    time: Res<Time<Virtual>>,
    mut finished: MessageReader<ManeuverFinished>,
    mut brains: Query<(&mut Brain, &Perception)>,
) {
    let now = time.elapsed_secs();
    for message in finished.read() {
        let Ok((mut brain, perception)) = brains.get_mut(message.entity) else {
            continue;
        };
        let cooldown = perception.maneuver_tuning[message.maneuver as usize]
            .map_or(0.0, |tuning| tuning.cooldown);
        brain.finish_maneuver(message.maneuver, now + cooldown);
    }
}

/// Scores the staged perception and writes the outcome back: the movement
/// layer's mode or maneuver, and the action intent.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &mut BrainRng, &Perception, &mut ActionIntent)>,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, perception, mut action_intent) in &mut brains {
        let definition = brain.definition;
        brain.mode = choose_mode(&mut brain, &mut rng, &perception.context, now);
        let actions = choose_actions(definition, &perception.context);
        if action_intent.0 != actions {
            action_intent.0 = actions;
        }
    }
}

/// Runs the movement layer and returns the mode to drive. A running maneuver
/// is committed: it is never re-scored and ends when its executor reports it,
/// or here when the target is gone. A winning maneuver rolls its start chance;
/// a miss waits out its cooldown and the layer picks again without it.
fn choose_mode(
    brain: &mut Brain,
    rng: &mut BrainRng,
    context: &DecisionContext,
    now: f32,
) -> MotionMode {
    if brain.maneuver().is_some() {
        if context.target.is_some() {
            return brain.mode;
        }
        brain.cancel_maneuver();
    }
    let mut declined = [false; Maneuver::ALL.len()];
    loop {
        let can_start = |movement: Movement| {
            context.can_start(movement)
                && !matches!(movement, Movement::Maneuver(maneuver) if declined[maneuver as usize])
        };
        match choose_movement(brain.definition, context, brain.mode, can_start) {
            Some(Movement::Mode(mode)) => return mode,
            None => return brain.mode,
            Some(Movement::Maneuver(maneuver)) => {
                let Some(tuning) = context.maneuver_tuning[maneuver as usize] else {
                    declined[maneuver as usize] = true;
                    continue;
                };
                if tuning.chance >= 1.0 || rng.range(0.0, 1.0) < tuning.chance {
                    brain.start_maneuver(maneuver);
                    return brain.mode;
                }
                brain.defer_maneuver(maneuver, now + tuning.cooldown);
                declined[maneuver as usize] = true;
            }
        }
    }
}
