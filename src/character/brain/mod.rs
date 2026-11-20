mod action;
pub(super) mod considerations;
mod decide;
#[cfg(feature = "dev_tools")]
mod diagnostics;
mod engine;
mod inputs;
mod issue_motion;
mod perception;
mod vocabulary;

use bevy::prelude::*;
use rand::RngExt;

pub use self::action::Action;
pub use self::decide::ScoreTrace;
use self::decide::{RunningBehavior, schedule_first_decision};
use self::engine::validate_brain;
pub(super) use self::engine::{CommitPolicy, Curve};
pub(super) use self::inputs::ConsiderationInput;
use self::inputs::DecisionInputs;
pub use self::issue_motion::Motion;
use self::perception::TargetLocationHint;
#[cfg(feature = "dev_tools")]
pub(crate) use self::perception::TargetLock;
pub(super) use self::perception::Watching;
#[cfg(feature = "dev_native")]
pub(crate) use self::perception::lock_onto_target;
pub(crate) use self::perception::{TargetInfo, Targeting, VisionCone};
pub use self::vocabulary::BrainDefinition;
pub(super) use self::vocabulary::{Behavior, Consideration};
use super::motion::{
    AssumedVelocity, HomeArea, MotionCommands, MotionFinished, MotionKey, MotionStatus,
};
use crate::character::npc::VILLAGER;
use crate::prelude::*;

/// Seconds between a brain's scheduled decisions.
const DECISION_PERIOD: f32 = 0.1;

/// A character's utility brain: its static definition, the behaviors running in its motion and
/// action slots, and its start roll retry delays. Only [`decide`] changes which behaviors it holds,
/// every [`DECISION_PERIOD`] with a random per-brain offset; the Execute systems note what they
/// issued. Its requests start and stop the game's motions and actions, which report back with
/// [`MotionFinished`], [`ActionFinished`] or [`ActionFailed`].
#[derive(Component, Debug)]
#[component(on_add = schedule_first_decision)]
pub struct Brain {
    definition: &'static BrainDefinition,
    running_behaviors: Vec<RunningBehavior>,
    start_roll_ready_at: Vec<f32>,
    attack_finish_pending: bool,
    next_decision_at: f32,
}

impl Brain {
    /// Starts `definition` with both slots empty.
    pub fn new(definition: &'static BrainDefinition) -> Self {
        Self {
            definition,
            running_behaviors: Vec::with_capacity(2),
            start_roll_ready_at: vec![0.0; definition.behaviors.len()],
            attack_finish_pending: false,
            next_decision_at: 0.0,
        }
    }

    pub fn definition(&self) -> &'static BrainDefinition {
        self.definition
    }

    /// The held motion; `None` while the slot is empty.
    pub fn motion(&self) -> Option<Motion> {
        self.current_motion_behavior().map(|(_, motion)| motion)
    }

    /// The held action, including a deliberate `Idle`; `None` while the slot is empty.
    pub fn action(&self) -> Option<Action> {
        self.running_behaviors
            .iter()
            .find_map(|running| running.action)
    }

    fn definition_of_running_behavior(&self, running: RunningBehavior) -> Behavior {
        self.definition.behaviors[running.behavior]
    }

    fn current_motion_behavior(&self) -> Option<(RunningBehavior, Motion)> {
        self.running_behaviors
            .iter()
            .find_map(|&running| running.motion.map(|motion| (running, motion)))
    }

    fn running_behavior_with(&self, action: Action) -> Option<RunningBehavior> {
        self.running_behaviors
            .iter()
            .copied()
            .find(|running| running.action == Some(action))
    }

    /// Whether any behavior with `motion` is in a start roll retry delay.
    fn is_start_roll_waiting(&self, motion: Motion, now: f32) -> bool {
        self.definition
            .behaviors
            .iter()
            .zip(&self.start_roll_ready_at)
            .any(|(behavior, &ready_at)| {
                now < ready_at && behavior.motion.is_some_and(|held| held == motion)
            })
    }

    /// Whether behavior `index` may start now: it is not in a start roll retry delay, its motion
    /// is ready (see [`MotionStatus::is_ready`]), and its action can run (see
    /// [`DecisionInputs::can_run`]). A committed behavior also needs exactly one part that ends on
    /// its own, since nothing else would end it, and a continuously scored one needs a motion that
    /// runs until replaced.
    fn is_available(&self, index: usize, inputs: &DecisionInputs) -> bool {
        let behavior = &self.definition.behaviors[index];
        if inputs.now < self.start_roll_ready_at[index] {
            return false;
        }

        let motion_available = |motion: Motion| match motion {
            Motion::Idle => true,
            Motion::Run(key) => inputs.is_motion_ready(key),
        };
        let action_available = |action: Action| match action {
            Action::Idle => true,
            Action::Use(_) => inputs.has_action_source(action) && inputs.can_run(action),
        };
        if !(behavior.motion.is_none_or(motion_available)
            && behavior.action.is_none_or(action_available))
        {
            return false;
        }

        let Behavior { motion, action, .. } = *behavior;
        let finite_motion = motion
            .and_then(Motion::key)
            .is_some_and(|key| inputs.motion_ends_on_its_own(key));
        let finite_action = action.is_some_and(|action| inputs.action_ends_on_its_own(action));
        if behavior.commit.is_committed() {
            let finite_parts = usize::from(finite_motion) + usize::from(finite_action);
            if finite_parts != 1 {
                warn_once!(
                    "a committed behavior over {motion:?} and {action:?} never runs: it has \
                     {finite_parts} finite parts"
                );
            }
            finite_parts == 1
        } else {
            if finite_motion {
                warn_once!(
                    "a continuously scored behavior over {motion:?} never runs: its motion is \
                     finite"
                );
            }
            !finite_motion
        }
    }

    /// An attack finished since the last decision.
    fn attack_finish_pending(&self) -> bool {
        self.attack_finish_pending
    }

    /// Notes an attack finish for the next decision to read.
    fn record_attack_finished(&mut self) {
        self.attack_finish_pending = true;
    }

    /// Puts the first decision at a random point in the next period, so brains spawned
    /// together don't all decide on the same frame.
    fn randomize_decision_phase(&mut self, now: f32) {
        self.next_decision_at = now + rand::rng().random_range(0.0..DECISION_PERIOD);
    }

    /// Whether a scheduled decision is due at `now`. A due brain moves to its next period
    /// boundary after `now`, keeping its phase without a burst of catch-up decisions after a
    /// long frame.
    fn advance_decision_clock(&mut self, now: f32) -> bool {
        if now < self.next_decision_at {
            return false;
        }
        let periods_passed = ((now - self.next_decision_at) / DECISION_PERIOD).floor() + 1.0;
        self.next_decision_at += periods_passed * DECISION_PERIOD;
        // Rounding near an exact multiple can land the boundary on `now`.
        if self.next_decision_at <= now {
            self.next_decision_at += DECISION_PERIOD;
        }
        true
    }

    fn running_behavior_ids(&self) -> Vec<usize> {
        self.running_behaviors
            .iter()
            .map(|running| running.behavior)
            .collect()
    }

    /// Applies a decision. The only place an attack-finished edge is consumed.
    /// `missed_start_ids` are the behaviors that won but failed their start roll.
    fn commit_decision(
        &mut self,
        winning_behavior_ids: &[usize],
        missed_start_ids: &[usize],
        inputs: &DecisionInputs,
    ) {
        self.assign_slots(winning_behavior_ids, inputs);
        for &index in missed_start_ids {
            if let Some(start_chance) = self.definition.behaviors[index].start_chance {
                self.start_roll_ready_at[index] = inputs.now + start_chance.retry_seconds;
            }
        }
        self.attack_finish_pending = false;
    }

    /// Runs the winning behaviors, keeping the record of one already running. A committed
    /// behavior keeps running until a winner shares one of its slots, which ends
    /// it; anything else not won again stops. Winners never share a slot, so
    /// neither do running behaviors.
    fn assign_slots(&mut self, winning_behavior_ids: &[usize], inputs: &DecisionInputs) {
        let behaviors = self.definition.behaviors;
        for running in self.running_behaviors.clone() {
            let displaced = winning_behavior_ids.iter().any(|&winning_behavior_id| {
                winning_behavior_id != running.behavior
                    && behaviors[winning_behavior_id]
                        .shares_a_slot_with(&behaviors[running.behavior])
            });
            if running.committed && displaced {
                self.drop_committed_behavior(running);
            }
        }
        let previous = std::mem::take(&mut self.running_behaviors);
        self.running_behaviors
            .extend(previous.iter().copied().filter(|running| {
                running.committed && !winning_behavior_ids.contains(&running.behavior)
            }));
        for &winning_behavior_id in winning_behavior_ids {
            let running = match previous
                .iter()
                .find(|running| running.behavior == winning_behavior_id)
            {
                Some(&kept) => kept,
                None => self.start_running_behavior(winning_behavior_id, inputs.now),
            };
            self.running_behaviors.push(running);
        }
    }

    /// Creates a running behavior for behavior `index` with nothing issued yet.
    fn start_running_behavior(&self, index: usize, now: f32) -> RunningBehavior {
        let behavior = self.definition.behaviors[index];
        RunningBehavior {
            behavior: index,
            motion: behavior.motion,
            action: behavior.action,
            motion_issued: false,
            action_issued: false,
            committed: behavior.commit.is_committed(),
            started_at: now,
        }
    }

    /// Drops each stale committed behavior; see
    /// [`RunningBehavior::should_drop_committed_behavior`].
    fn drop_stale_committed_behaviors(&mut self, inputs: &DecisionInputs) {
        for running in self.running_behaviors.clone() {
            let behavior = self.definition_of_running_behavior(running);
            let read = |input| inputs.read_consideration_input(input, self);
            if running.should_drop_committed_behavior(&behavior, &read, inputs) {
                self.drop_committed_behavior(running);
            }
        }
    }

    fn drop_committed_behavior(&mut self, running: RunningBehavior) {
        self.running_behaviors
            .retain(|other| other.behavior != running.behavior);
    }

    fn record_issued_motion(&mut self, behavior: usize) {
        self.running_behavior_mut(behavior).motion_issued = true;
    }

    fn record_issued_action(&mut self, behavior: usize) {
        self.running_behavior_mut(behavior).action_issued = true;
    }

    fn running_behavior_mut(&mut self, behavior: usize) -> &mut RunningBehavior {
        self.running_behaviors
            .iter_mut()
            .find(|running| running.behavior == behavior)
            .expect("issued behavior is not running")
    }

    /// Ends the committed behavior that issued what finished. Only a finish of something a behavior
    /// issued ends it, so a stale finish clears nothing and returns `false`.
    fn finish_committed_behavior(
        &mut self,
        issued_what_finished: impl Fn(RunningBehavior) -> bool,
    ) -> bool {
        let Some(position) = self
            .running_behaviors
            .iter()
            .position(|&running| running.committed && issued_what_finished(running))
        else {
            return false;
        };
        self.running_behaviors.remove(position);
        true
    }
}

/// Ordered brain phases. Perception feeds targeting, aim, decisions, and execution of those
/// decisions.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum BrainSystems {
    Perception,
    Targeting,
    Aim,
    Decision,
    Execute,
}

pub(super) fn plugin(app: &mut App) {
    for &kind in EnemyKind::ALL {
        if let Err(error) = validate_brain(kind.brain()) {
            panic!("{kind:?} brain is invalid: {error}");
        }
    }
    if let Err(error) = validate_brain(&VILLAGER) {
        panic!("villager brain is invalid: {error}");
    }

    #[cfg(feature = "dev_tools")]
    app.add_plugins(diagnostics::plugin);

    app.configure_sets(
        Update,
        (
            (
                BrainSystems::Perception,
                BrainSystems::Targeting,
                BrainSystems::Aim,
            )
                .chain()
                .in_set(SimulationSystems::Perceive),
            (BrainSystems::Decision, BrainSystems::Execute)
                .chain()
                .in_set(SimulationSystems::Act),
        ),
    );

    app.add_plugins((perception::plugin, action::plugin))
        .add_observer(decide::on_motion_finished)
        .add_observer(decide::on_action_finished)
        .add_observer(decide::on_action_failed)
        .add_systems(
            Update,
            (
                decide::decide_behavior.in_set(BrainSystems::Decision),
                issue_motion::issue_running_motion.in_set(BrainSystems::Execute),
            ),
        );
}
