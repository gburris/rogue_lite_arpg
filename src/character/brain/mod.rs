mod considerations;
mod decide;
#[cfg(feature = "dev_tools")]
mod diagnostics;
mod engine;
mod inputs;
mod perception;

use bevy::prelude::*;
use rand::RngExt;

pub(super) use self::considerations::{
    ACTION_BASELINE_IDLE, ATTACK_FINISHED, Behavior, CAST_DISTANCE_RANGE, CAST_HYSTERESIS,
    CAST_MAX_RANGE, CAST_MIN_RANGE, CHASE_CAST_RANGE, Consideration, HAS_TARGET, IN_MAINHAND_REACH,
    INSIDE_OUTER_CAST_EDGE, LINE_OF_SIGHT, MAINHAND, NO_TARGET, RETURN_TO_HOME, SEEK_LOST_TARGET,
    WANDER, consideration, distance_threshold, flag, flag_kept_while_current, mana_at_or_below,
    motion_running_for, reach_fraction_threshold, target_speed_at_most, target_within_distance,
    within_reach,
};
pub use self::considerations::{Action, BrainDefinition};
pub use self::decide::ScoreTrace;
use self::decide::{RunningBehavior, schedule_first_decision};
use self::engine::validate_brain;
pub(super) use self::engine::{CommitPolicy, Curve};
pub(super) use self::inputs::ConsiderationInput;
use self::inputs::DecisionInputs;
#[cfg(feature = "dev")]
pub use self::perception::TargetInfo;
use self::perception::TargetLocationHint;
#[cfg(feature = "dev_tools")]
pub use self::perception::TargetLock;
#[cfg(feature = "dev_tools")]
pub use self::perception::Targeting;
pub use self::perception::VisionCone;
pub(super) use self::perception::Watching;
#[cfg(feature = "dev_native")]
pub use self::perception::lock_onto_target;
use super::motion::{
    Home, HomeArea, MotionCommands, MotionFailed, MotionFinished, MotionKey, MotionStatus,
};
use crate::character::npc::VILLAGER;
use crate::prelude::*;

/// Seconds between a brain's scheduled decisions.
const DECISION_PERIOD: f32 = 0.1;

/// A character's utility brain: its static definition, the behaviors running in its motion and
/// action slots, and its start roll retry delays. Only [`decide`] changes which behaviors it holds,
/// every [`DECISION_PERIOD`] with a random per-brain offset. Its requests start and stop the
/// game's motions and actions, which report back with
/// [`MotionFinished`], [`MotionFailed`], [`ActionFinished`] or [`ActionFailed`].
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
    pub fn motion(&self) -> Option<MotionKey> {
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

    fn current_motion_behavior(&self) -> Option<(RunningBehavior, MotionKey)> {
        self.running_behaviors
            .iter()
            .find_map(|&running| running.motion.map(|motion| (running, motion)))
    }

    /// Whether any behavior with `motion` is in a start roll retry delay.
    fn is_start_roll_waiting(&self, motion: MotionKey, now: f32) -> bool {
        self.definition
            .behaviors
            .iter()
            .zip(&self.start_roll_ready_at)
            .any(|(behavior, &ready_at)| {
                now < ready_at && behavior.motion.is_some_and(|held| held == motion)
            })
    }

    /// Whether behavior `index` may start now: it is not in a start roll retry delay, its motion
    /// can start (see [`MotionStatus::can_start`]), its action can run (see
    /// [`DecisionInputs::can_run`]), and it suits its commit policy (see
    /// [`Behavior::suits_commit_policy`]).
    fn is_available(&self, index: usize, inputs: &DecisionInputs) -> bool {
        let behavior = &self.definition.behaviors[index];
        if inputs.now < self.start_roll_ready_at[index] {
            return false;
        }

        let motion_available = |key: MotionKey| inputs.can_start_motion(key);
        let action_available = |action: Action| match action {
            Action::Idle => true,
            Action::Use(_) => inputs.has_action_source(action) && inputs.can_run(action),
        };
        behavior.motion.is_none_or(motion_available)
            && behavior.action.is_none_or(action_available)
            && behavior.suits_commit_policy(inputs)
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

    /// Creates a running behavior for behavior `index`.
    fn start_running_behavior(&self, index: usize, now: f32) -> RunningBehavior {
        let behavior = self.definition.behaviors[index];
        RunningBehavior {
            behavior: index,
            motion: behavior.motion,
            action: behavior.action,
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

    /// Ends the committed behavior whose held motion or action finished. A stale finish
    /// matches no behavior, so it clears nothing and returns `false`.
    fn finish_committed_behavior(
        &mut self,
        ran_what_finished: impl Fn(RunningBehavior) -> bool,
    ) -> bool {
        let Some(position) = self
            .running_behaviors
            .iter()
            .position(|&running| running.committed && ran_what_finished(running))
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

    app.add_plugins(perception::plugin)
        .add_observer(decide::on_motion_finished)
        .add_observer(decide::on_motion_failed)
        .add_observer(decide::on_action_finished)
        .add_observer(decide::on_action_failed)
        .add_systems(
            Update,
            (
                (decide::check_new_brains, decide::decide_behavior)
                    .chain()
                    .in_set(BrainSystems::Decision),
                (decide::issue_running_motion, decide::issue_running_action)
                    .in_set(BrainSystems::Execute),
            ),
        );
}
