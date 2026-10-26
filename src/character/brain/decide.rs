//! The decision pipeline: observe the world, release or abort a stale lock,
//! score, and commit. This is the only code that changes which behaviors a
//! [`Brain`] holds.

use bevy::prelude::*;

use super::capability::{Capabilities, CapabilityComponents, build_capabilities};
use super::model::Behavior;
use super::motion::{MotionAborted, MotionFinished};
use super::scoring::assign_behaviors_to_slots;
use super::senses::{Observation, ObservedComponents, WorldUseChecks, observe_world_facts};
use super::{Action, ActionAborted, ActionFinished, Brain, BrainRng, MotionActionTable};

type DecidingBrains<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut Brain,
        &'static mut BrainRng,
        ObservedComponents,
        CapabilityComponents,
    ),
>;

/// Marks a locked motion's part finished and starts its cooldown. Once every
/// finite part of the lock is done it releases, and the entity decides again
/// at once instead of waiting for the next tick.
pub(super) fn release_finished_motion(
    finished: On<MotionFinished>,
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, observed, components)) = brains.get_mut(finished.entity) else {
        return;
    };
    let capabilities = build_capabilities(&components, &use_checks.equipment);
    let Some(released) = brain.finish_motion_part(finished.motion, now) else {
        return;
    };
    let cooldown = capabilities[finished.motion].map_or(0.0, |tuning| tuning.cooldown);
    brain.start_cooldown(finished.motion, now + cooldown);
    if released {
        let observation = observe_world_facts(&observed, &capabilities, &use_checks);
        decide_slot_assignments(&mut brain, &mut rng, &observation, &capabilities, now);
    }
}

/// Drops a finite motion its executor could not start, with no cooldown, and
/// decides again for that entity at once.
pub(super) fn abort_failed_motion(
    aborted: On<MotionAborted>,
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let Ok((mut brain, mut rng, observed, components)) = brains.get_mut(aborted.entity) else {
        return;
    };
    if !brain
        .active_motion()
        .is_some_and(|active| active.motion == aborted.motion)
    {
        return;
    }
    brain.abort_locked_behavior();
    let capabilities = build_capabilities(&components, &use_checks.equipment);
    let observation = observe_world_facts(&observed, &capabilities, &use_checks);
    decide_slot_assignments(
        &mut brain,
        &mut rng,
        &observation,
        &capabilities,
        time.elapsed_secs(),
    );
}

/// Drops a locked action its applier could not start, with no cooldown, and
/// decides again for that entity at once.
pub(super) fn abort_failed_action(
    aborted: On<ActionAborted>,
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let Ok((mut brain, mut rng, observed, components)) = brains.get_mut(aborted.entity) else {
        return;
    };
    if brain.active_action().is_none() {
        return;
    }
    brain.abort_locked_behavior();
    let capabilities = build_capabilities(&components, &use_checks.equipment);
    let observation = observe_world_facts(&observed, &capabilities, &use_checks);
    decide_slot_assignments(
        &mut brain,
        &mut rng,
        &observation,
        &capabilities,
        time.elapsed_secs(),
    );
}

/// Starts an ability's cooldown when its use finishes, and marks a locked
/// action's part finished. Once every finite part of the lock is done it
/// releases and the entity decides again at once. An equipment finish needs no
/// flag: the decision reads it from the observed attack count.
pub(super) fn finish_action(
    finished: On<ActionFinished>,
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    let Ok((mut brain, mut rng, observed, components)) = brains.get_mut(finished.entity) else {
        return;
    };
    let capabilities = build_capabilities(&components, &use_checks.equipment);
    if let Some(Action::Ability(ability)) = finished.action {
        let cooldown = capabilities[ability].map_or(0.0, |tuning| tuning.cooldown);
        brain.start_cooldown(ability, now + cooldown);
    }
    if brain.finish_action_part(finished.action, now) == Some(true) {
        let observation = observe_world_facts(&observed, &capabilities, &use_checks);
        decide_slot_assignments(&mut brain, &mut rng, &observation, &capabilities, now);
    }
}

/// Observes every brain's world and fills its motion and action slots.
pub(super) fn decide_behavior(
    time: Res<Time<Virtual>>,
    use_checks: WorldUseChecks,
    mut brains: DecidingBrains,
) {
    let now = time.elapsed_secs();
    for (mut brain, mut rng, observed, components) in &mut brains {
        let capabilities = build_capabilities(&components, &use_checks.equipment);
        let observation = observe_world_facts(&observed, &capabilities, &use_checks);
        decide_slot_assignments(&mut brain, &mut rng, &observation, &capabilities, now);
    }
}

/// Fills the free slots. A locked behavior is never re-scored: it ends when
/// its executor reports it or the lock is aborted. A winning locked motion
/// rolls its start chance; a miss waits out its cooldown and the next best
/// behavior takes the slot. A slot nothing claims keeps its holder.
fn decide_slot_assignments(
    brain: &mut Brain,
    rng: &mut BrainRng,
    observation: &Observation,
    capabilities: &Capabilities,
    now: f32,
) {
    brain.release_or_abort_lock(observation, now);
    let (context, availability) = brain.build_scoring_inputs(observation, capabilities, now);
    let definition = brain.definition();
    let mut missed_start_cooldowns = MotionActionTable::<Option<f32>>::default();
    let assignment = assign_behaviors_to_slots(
        definition,
        &context,
        &availability,
        brain.holders(),
        |index| {
            passes_start_roll(
                rng,
                capabilities,
                definition.behaviors[index].value,
                now,
                &mut missed_start_cooldowns,
            )
        },
    );
    brain.commit_decision(
        assignment,
        &missed_start_cooldowns,
        observation.attacks_finished,
        now,
    );
}

/// Whether a winning behavior actually starts: a finite motion rolls its
/// chance, and a miss records when its cooldown ends.
fn passes_start_roll(
    rng: &mut BrainRng,
    capabilities: &Capabilities,
    behavior: Behavior,
    now: f32,
    missed_start_cooldowns: &mut MotionActionTable<Option<f32>>,
) -> bool {
    let Some(motion) = behavior.motion.filter(|motion| motion.is_finite()) else {
        return true;
    };
    let Some(tuning) = capabilities[motion] else {
        return false;
    };
    if tuning.chance >= 1.0 || rng.range(0.0, 1.0) < tuning.chance {
        return true;
    }
    missed_start_cooldowns[motion] = Some(now + tuning.cooldown);
    false
}
