//! The decision tick: scores the staged perception and writes intents. This is
//! the only code that changes a [`Brain`]'s mode or maneuver.

use bevy::prelude::*;

use super::scoring::{commit_for, decide_with_arbiter};
use super::senses::Perception;
use super::{
    AbilityIntent, Brain, BrainDecision, BrainDefinition, BrainRng, DecisionContext,
    MIN_DWELL_SECS, ManeuverFinished, MotionMode, SlotIntent,
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
        let cooldown = perception.self_state.maneuver_tuning[message.maneuver as usize]
            .map_or(0.0, |tuning| tuning.cooldown);
        brain.finish_maneuver(message.maneuver, now + cooldown);
    }
}

/// Scores the staged perception and writes the outcome back: locomotion mode,
/// dwell and commit timers, the maneuver, and the intent components.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    mut brains: Query<(
        &mut Brain,
        &mut BrainRng,
        &Perception,
        &mut AbilityIntent,
        Option<&mut SlotIntent>,
    )>,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, perception, mut ability_intent, mut slot_intent) in &mut brains {
        let definition = brain.definition;
        // The utility proposal is memoryless; the arbiter answers "is now a
        // good time to actually switch?" Same-group jitter waits out the dwell
        // or loses on the challenger margin, while group changes and an
        // ineligible incumbent go through at once.
        let (decision, accepted) = decide_with_arbiter(
            definition,
            &perception.context,
            brain.mode,
            brain.dwell_until - now,
        );
        settle_mode(&mut brain, &mut rng, accepted, definition, now);
        advance_maneuver(&mut brain, &mut rng, &perception.context, &decision, now);
        publish_intents(slot_intent.as_deref_mut(), &mut ability_intent, &decision);
    }
}

fn settle_mode(
    brain: &mut Brain,
    rng: &mut BrainRng,
    accepted: MotionMode,
    definition: &BrainDefinition,
    now: f32,
) {
    let changed = accepted != brain.mode;
    if changed {
        brain.mode = accepted;
        brain.dwell_until = now + MIN_DWELL_SECS;
    }
    // Arm the calm-cycle window on entry, and once on spawn when the
    // initial mode is already calm. An expired window is not re-armed
    // while the mode holds: expiry is what asks the phase to advance.
    let unarmed_calm = accepted.is_calm() && brain.committed_until <= 0.0;
    if changed || unarmed_calm {
        brain.committed_until = commit_for(definition, accepted)
            .map_or(now, |window| now + rng.range(window.min(), window.max()));
    }
    // A reactive mode hands the calm cycle back to idle, which is where the
    // cycle always starts.
    brain.calm_phase = Some(if accepted.is_calm() {
        accepted
    } else {
        MotionMode::Idle
    });
}

/// Starts, keeps or cancels the maneuver. A running maneuver is exclusive and
/// ends when its executor reports it, or here when the target is gone. A
/// ready candidate then rolls its chance; a miss waits out the cooldown.
fn advance_maneuver(
    brain: &mut Brain,
    rng: &mut BrainRng,
    context: &DecisionContext,
    decision: &BrainDecision,
    now: f32,
) {
    if brain.maneuver().is_some() {
        if context.target.is_none() {
            brain.cancel_maneuver();
        }
        return;
    }
    let Some(candidate) = decision.maneuver else {
        return;
    };
    let Some(tuning) = context.self_state.maneuver_tuning[candidate as usize] else {
        return;
    };
    if tuning.chance >= 1.0 || rng.range(0.0, 1.0) < tuning.chance {
        brain.start_maneuver(candidate);
    } else {
        brain.defer_maneuver(candidate, now + tuning.cooldown);
    }
}

fn publish_intents(
    slot_intent: Option<&mut SlotIntent>,
    ability_intent: &mut AbilityIntent,
    decision: &BrainDecision,
) {
    if let Some(slot_intent) = slot_intent
        && slot_intent.0 != decision.slot_intents
    {
        slot_intent.set_all(decision.slot_intents);
    }
    if ability_intent.0 != decision.abilities {
        ability_intent.set_all(decision.abilities);
    }
}
