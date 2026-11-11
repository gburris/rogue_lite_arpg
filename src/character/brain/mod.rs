mod action;
mod capability_profile;
pub(super) mod considerations;
mod cooldowns;
mod decide;
#[cfg(feature = "dev_tools")]
mod diagnostics;
mod engine;
mod inputs;
mod motion;
mod perception;
mod running_state;
mod schedule;

use bevy::prelude::*;
use rand::RngExt;

pub use self::action::{Ability, Action, Bite, Breath, Heal};
use self::action::{ActionFinished, ActionOutcome, StartedEquipmentUses};
use self::capability_profile::CapabilityProfile;
use self::cooldowns::{CooldownKey, Cooldowns};
pub use self::decide::ScoreTrace;
use self::decide::schedule_first_decision;
use self::engine::validate_brain;
pub(super) use self::engine::{Behavior, CommitPolicy, Consideration, Curve};
pub use self::engine::{BrainDefinition, Interval};
pub(super) use self::inputs::ConsiderationInput;
use self::inputs::DecisionInputs;
use self::motion::Steering;
pub use self::motion::{Dash, HomeArea, Motion, Orbit, Swoop, TurnRate, Wander};
#[cfg(feature = "dev_tools")]
pub(crate) use self::perception::TargetLock;
pub(super) use self::perception::Watching;
pub(crate) use self::perception::{TargetInfo, Targeting, VisionCone};
use self::running_state::RunningBehavior;
use self::schedule::BrainSystems;
use crate::character::npc::VILLAGER;
use crate::prelude::*;

/// Seconds between a brain's scheduled decisions.
const DECISION_PERIOD: f32 = 0.1;

/// A character's utility brain: its static definition, the behaviors running
/// in its motion and action slots, and its cooldowns. Only [`decide`] writes it,
/// at 10 Hz on a random per-brain phase; motion and action systems read it
/// and answer with [`MotionFinished`](motion::MotionFinished) or
/// [`ActionFinished`].
#[derive(Component, Debug)]
#[require(StartedEquipmentUses, Steering, Kinematics)]
#[component(on_add = schedule_first_decision)]
pub struct Brain {
    definition: &'static BrainDefinition,
    running_behaviors: Vec<RunningBehavior>,
    next_behavior_run_id: u32,
    cooldowns: Cooldowns,
    attack_finish_pending: bool,
    next_decision_at: f32,
}

impl Brain {
    /// Starts `definition` with both slots empty.
    pub fn new(definition: &'static BrainDefinition) -> Self {
        Self {
            definition,
            running_behaviors: Vec::with_capacity(2),
            next_behavior_run_id: 0,
            cooldowns: Cooldowns::default(),
            attack_finish_pending: false,
            next_decision_at: 0.0,
        }
    }

    pub fn definition(&self) -> &'static BrainDefinition {
        self.definition
    }

    /// The held motion; `None` while the slot is empty.
    pub fn motion(&self) -> Option<Motion> {
        self.running_behaviors
            .iter()
            .find_map(|running| running.motion)
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

    fn current_motion_behavior(&self) -> Option<RunningBehavior> {
        self.running_behaviors
            .iter()
            .copied()
            .find(|running| running.motion.is_some())
    }

    fn held_action(&self, action: Action) -> Option<RunningBehavior> {
        self.running_behaviors
            .iter()
            .copied()
            .find(|running| running.action == Some(action))
    }

    fn is_cooldown_ready(&self, key: impl CooldownKey, now: f32) -> bool {
        self.cooldowns.is_ready(key, now)
    }

    fn start_cooldown(&mut self, key: impl CooldownKey, ready_at: f32) {
        self.cooldowns.start(key, ready_at);
    }

    /// Starts `motion`'s cooldown for as long as its capability profile says.
    fn start_motion_cooldown(&mut self, motion: Motion, inputs: &DecisionInputs) {
        let cooldown = inputs
            .motion_profile(motion)
            .map_or(0.0, |profile| profile.cooldown);
        self.start_cooldown(motion, inputs.now + cooldown);
    }

    /// Starts `ability`'s cooldown for as long as its capability profile says.
    fn start_ability_cooldown(&mut self, ability: Ability, inputs: &DecisionInputs) {
        let cooldown = inputs
            .action_profile(Action::Ability(ability))
            .map_or(0.0, |profile| profile.cooldown);
        self.start_cooldown(ability, inputs.now + cooldown);
    }

    /// Whether `behavior` may start now: each part's capability is owned, off its
    /// cooldown, and allowed by the world.
    fn is_available(&self, behavior: &Behavior, inputs: &DecisionInputs) -> bool {
        let motion_available = |motion: Motion| {
            inputs.motion_profile(motion).is_some() && self.is_cooldown_ready(motion, inputs.now)
        };
        let action_available = |action: Action| {
            inputs.action_profile(action).is_some()
                && self.is_cooldown_ready(action, inputs.now)
                && inputs.world_allows_use(action)
        };
        behavior.motion.is_none_or(motion_available) && behavior.action.is_none_or(action_available)
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
    /// `missed_starts` are the finite motions that won but failed their start roll.
    fn commit_decision(
        &mut self,
        winning_behavior_ids: &[usize],
        missed_starts: &[Motion],
        inputs: &DecisionInputs,
    ) {
        self.assign_slots(winning_behavior_ids, inputs);
        for &motion in missed_starts {
            self.start_motion_cooldown(motion, inputs);
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
                self.drop_committed_behavior(running, inputs);
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

    /// Stamps a new run of behavior `index` with the next run id.
    fn start_running_behavior(&mut self, index: usize, now: f32) -> RunningBehavior {
        self.next_behavior_run_id = self.next_behavior_run_id.wrapping_add(1);
        let behavior = self.definition.behaviors[index];
        RunningBehavior {
            behavior: index,
            motion: behavior.motion,
            action: behavior.action,
            behavior_run_id: self.next_behavior_run_id,
            committed: behavior.commit.is_committed(),
            started_at: now,
        }
    }

    /// Drops each committed behavior that lost its target, was stunned or timed out.
    fn drop_stale_committed_behaviors(&mut self, inputs: &DecisionInputs) {
        for running in self.running_behaviors.clone() {
            if running.should_drop_committed_behavior(inputs) {
                self.drop_committed_behavior(running, inputs);
            }
        }
    }

    /// Ends `running` and starts its finite part's cooldown so it can't
    /// restart next tick. Equipment keeps its own cooldown, so a use starts none.
    fn drop_committed_behavior(&mut self, running: RunningBehavior, inputs: &DecisionInputs) {
        self.running_behaviors
            .retain(|other| other.behavior != running.behavior);
        let behavior = self.definition_of_running_behavior(running);
        match (behavior.motion, behavior.action) {
            (Some(motion), _) if motion.is_finite() => self.start_motion_cooldown(motion, inputs),
            (_, Some(Action::Ability(ability))) => self.start_ability_cooldown(ability, inputs),
            _ => {}
        }
    }

    /// Ends the committed behavior stamped `behavior_run_id`; `false` for a stale finish.
    fn finish_committed_behavior(&mut self, behavior_run_id: u32) -> bool {
        let Some(position) = self
            .running_behaviors
            .iter()
            .position(|running| running.committed && running.behavior_run_id == behavior_run_id)
        else {
            return false;
        };
        self.running_behaviors.remove(position);
        true
    }
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
            (
                BrainSystems::Decision,
                BrainSystems::Execute,
                BrainSystems::Motion,
            )
                .chain()
                .in_set(SimulationSystems::Act),
        ),
    );

    app.add_plugins((perception::plugin, action::plugin, motion::plugin))
        .add_observer(decide::on_release_finished_motion)
        .add_observer(decide::on_finish_action)
        .add_systems(
            Update,
            decide::decide_behavior.in_set(BrainSystems::Decision),
        );
}
