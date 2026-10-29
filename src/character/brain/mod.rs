//! ECS glue for the pure utility brains. Data flows one way, one writer each:
//! 1. [`perception`] builds a [`DecisionContext`] from the world.
//! 2. [`decide`] runs the pure [`engine`] scoring and writes which behavior
//!    holds each slot on the [`Brain`].
//! 3. [`motion`] and [`action`] carry out the held motion and action and
//!    report [`MotionFinished`](motion::MotionFinished) or [`ActionFinished`].
//!
//! To add a continuous motion:
//! 1. [`motion`]: a `Motion` variant only for a new steering contract. A
//!    stateless one gets an arm in `direct::steer_direct_motion`; one with
//!    state gets a file with its component and an `apply_` system registered
//!    in `motion::plugin`, and is skipped in `apply_direct_steering`. A
//!    component the motion needs implements `Capability`, is recorded in
//!    `capability::build_capability_profiles`, and is listed in `capability_keys`.
//! 2. [`DecisionContext`] fields built in [`perception`], plus
//!    [`ConsiderationInput`](perception::ConsiderationInput) variants read in
//!    `read_consideration_input`.
//! 3. Behaviors over it in `character/enemy` and `character/npc`.
//!
//! A finite [`Motion`] needs a variant, a `Capability` component whose
//! `apply_` system follows its run with a `CurrentBehaviorRunId` and reports
//! `MotionFinished`, the same registration and capability listing, and a
//! committed behavior over it. A new [`Ability`] needs a variant, a `Capability`
//! component in `action::ability` listed in `AbilityCapabilities`, an applier,
//! and behaviors over it.

mod action;
mod capability;
pub(crate) mod considerations;
mod decide;
mod engine;
mod motion;
mod perception;

pub use action::{Ability, Action, Bite, Breath, Heal};
pub use capability::CapabilityProfile;
pub(crate) use engine::{Behavior, CommitPolicy, Consideration, Curve};
pub use engine::{BrainDefinition, Interval};
pub use motion::{Dash, HomeArea, Motion, Orbit, Swoop, TurnRate, Wander};
pub(crate) use perception::{ConsiderationInput, Targeting};
pub(super) use perception::{VisionCone, Watching};

use self::action::StartedEquipmentUses;
use self::capability::{
    AvailableMotionActionTable, Capability, CapabilityKey, CapabilityProfiles, MotionActionKey,
    MotionActionTable, capability_keys,
};
use self::decide::{decide_behavior, finish_action, release_finished_motion};
use self::engine::{
    BehaviorScoreExplanation, SlotAssignment, SlotHolder, SlotHolders, validate_brain,
};
use self::motion::Steering;
use self::perception::{DecisionContext, MemoryView};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{EnemyKind, tick_equippable_cooldown},
    world::MapRng,
};

use super::AiSet;
use super::npc::VILLAGER;

/// Longest a committed behavior may run, a safety net against a missing finish.
const COMMIT_TIMEOUT: f32 = 3.0;

/// A behavior that won a slot, with its motion and action copied from the
/// definition and stamped with a run id so a finite motion's
/// [`CurrentBehaviorRunId`](motion::CurrentBehaviorRunId) can tell a fresh
/// start from the run it just finished. A bundle holds both slots, one record
/// each. Appliers report through it, so the committed/continuously scored
/// finish policy lives here.
#[derive(Clone, Copy, Debug)]
struct RunningBehavior {
    behavior: usize,
    motion: Option<Motion>,
    action: Option<Action>,
    behavior_run_id: u32,
    committed: bool,
    started_at: f32,
}

impl RunningBehavior {
    fn report_completed(self, commands: &mut Commands, entity: Entity) {
        if let Some(finished) = self.build_action_finished(entity, ActionOutcome::Completed) {
            commands.trigger(finished);
        }
    }

    /// Ends a committed use the live world refused; a continuously scored use waits.
    fn report_not_started(self, commands: &mut Commands, entity: Entity) {
        if self.committed
            && let Some(finished) = self.build_action_finished(entity, ActionOutcome::NotStarted)
        {
            commands.trigger(finished);
        }
    }

    /// `None` for a record with no action to report on.
    fn build_action_finished(
        self,
        entity: Entity,
        outcome: ActionOutcome,
    ) -> Option<ActionFinished> {
        Some(ActionFinished {
            entity,
            action: self.action?,
            behavior_run_id: self.behavior_run_id,
            outcome,
        })
    }

    fn should_drop_committed_behavior(&self, context: &DecisionContext, now: f32) -> bool {
        self.committed
            && (context.target_view.is_none()
                || context.self_view.stunned
                || now - self.started_at > COMMIT_TIMEOUT)
    }
}

/// A character's utility brain: its static definition, the behaviors holding
/// its motion and action slots, and its cooldowns. Only [`decide`] writes it,
/// at 10 Hz on a random per-brain phase; motion and action systems read it
/// and answer with [`MotionFinished`](motion::MotionFinished) or
/// [`ActionFinished`].
#[derive(Component, Debug)]
#[require(BrainRng, StartedEquipmentUses, Steering)]
#[component(on_add = seed_brain_rng_and_schedule_first_decision)]
pub struct Brain {
    definition: &'static BrainDefinition,
    motion_slot: Option<RunningBehavior>,
    action_slot: Option<RunningBehavior>,
    next_behavior_run_id: u32,
    ready_at: MotionActionTable<f32>,
    attack_finish_pending: bool,
    next_decision_at: f32,
}

impl Brain {
    /// Starts `definition` with both slots empty.
    pub fn new(definition: &'static BrainDefinition) -> Self {
        Self {
            definition,
            motion_slot: None,
            action_slot: None,
            next_behavior_run_id: 0,
            ready_at: MotionActionTable::default(),
            attack_finish_pending: false,
            next_decision_at: 0.0,
        }
    }

    pub fn definition(&self) -> &'static BrainDefinition {
        self.definition
    }

    /// The held motion; `None` while the slot is empty.
    pub fn motion(&self) -> Option<Motion> {
        self.motion_slot.and_then(|running| running.motion)
    }

    /// The held action, including a deliberate `Idle`; `None` while the slot is empty.
    pub fn action(&self) -> Option<Action> {
        self.action_slot.and_then(|running| running.action)
    }

    fn definition_of_running_behavior(&self, running: RunningBehavior) -> Behavior {
        self.definition.behaviors[running.behavior]
    }

    fn current_motion_behavior(&self) -> Option<RunningBehavior> {
        self.motion_slot.filter(|running| running.motion.is_some())
    }

    fn held_action(&self, action: Action) -> Option<RunningBehavior> {
        self.action_slot
            .filter(|running| running.action == Some(action))
    }

    fn is_cooldown_ready(&self, key: impl MotionActionKey, now: f32) -> bool {
        now >= self.ready_at[key]
    }

    fn start_cooldown(&mut self, key: impl MotionActionKey, ready_at: f32) {
        self.ready_at[key] = ready_at;
    }

    fn slot_holders(&self) -> SlotHolders {
        let holder = |running: RunningBehavior| SlotHolder {
            behavior: running.behavior,
            interrupt_tier: self
                .definition_of_running_behavior(running)
                .commit
                .interrupt_tier(),
        };
        SlotHolders {
            motion: self.motion_slot.map(holder),
            action: self.action_slot.map(holder),
        }
    }

    /// Fills in the brain's memory on `context_without_memory`, and marks each
    /// capability available when owned, off cooldown and use-eligible.
    fn build_scoring_inputs(
        &self,
        context_without_memory: DecisionContext,
        capability_profiles: &CapabilityProfiles,
        now: f32,
    ) -> (DecisionContext, AvailableMotionActionTable) {
        let context = DecisionContext {
            memory_view: MemoryView {
                motion: self.motion(),
                attack_finished: self.attack_finish_pending,
            },
            ..context_without_memory
        };
        let mut available = MotionActionTable::splat(true);
        for key in capability_keys() {
            available[key] = capability_profiles[key].is_some()
                && self.is_cooldown_ready(key, now)
                && context.use_eligibility[key];
        }
        (context, AvailableMotionActionTable(available))
    }

    /// Applies a decision. The only place an attack-finished edge is consumed.
    fn commit_decision(
        &mut self,
        assignment: SlotAssignment,
        missed_start_cooldowns: &MotionActionTable<Option<f32>>,
        capability_profiles: &CapabilityProfiles,
        now: f32,
    ) {
        self.assign_slots(assignment, capability_profiles, now);
        for key in capability_keys() {
            if let Some(ready_at) = missed_start_cooldowns[key] {
                self.start_cooldown(key, ready_at);
            }
        }
        self.attack_finish_pending = false;
    }

    /// A slot nothing claimed empties unless a committed behavior holds it. A
    /// committed behavior that loses either slot ends in both.
    fn assign_slots(
        &mut self,
        assignment: SlotAssignment,
        capability_profiles: &CapabilityProfiles,
        now: f32,
    ) {
        for (holder, winner) in [
            (self.motion_slot, assignment.motion),
            (self.action_slot, assignment.action),
        ] {
            if let Some(displaced) = holder.filter(|holder| {
                holder.committed && winner.is_some_and(|winner| winner != holder.behavior)
            }) {
                self.drop_committed_behavior(displaced, capability_profiles, now);
            }
        }
        match assignment.motion {
            Some(index) => {
                self.motion_slot = self.start_or_keep_running_behavior(self.motion_slot, index, now)
            }
            None if self.motion_slot.is_none_or(|running| !running.committed) => {
                self.motion_slot = None
            }
            None => {}
        }
        match assignment.action {
            Some(index) => {
                self.action_slot = self.start_or_keep_running_behavior(self.action_slot, index, now)
            }
            None if self.action_slot.is_none_or(|running| !running.committed) => {
                self.action_slot = None
            }
            None => {}
        }
    }

    /// Keeps the current record if `index` already holds the slot, else stamps a new run id.
    fn start_or_keep_running_behavior(
        &mut self,
        current: Option<RunningBehavior>,
        index: usize,
        now: f32,
    ) -> Option<RunningBehavior> {
        if current.is_some_and(|running| running.behavior == index) {
            return current;
        }
        self.next_behavior_run_id = self.next_behavior_run_id.wrapping_add(1);
        let behavior = self.definition.behaviors[index];
        Some(RunningBehavior {
            behavior: index,
            motion: behavior.motion,
            action: behavior.action,
            behavior_run_id: self.next_behavior_run_id,
            committed: behavior.commit.is_committed(),
            started_at: now,
        })
    }

    /// Ends `running` in every slot and starts its finite part's cooldown so it
    /// can't restart next tick. Equipment keeps its own cooldown, so a use starts none.
    fn drop_committed_behavior(
        &mut self,
        running: RunningBehavior,
        capability_profiles: &CapabilityProfiles,
        now: f32,
    ) {
        self.release_behavior_from_slots(running.behavior);
        let behavior = self.definition_of_running_behavior(running);
        let finite_part = match (behavior.motion, behavior.action) {
            (Some(motion), _) if motion.is_finite() => CapabilityKey::Motion(motion),
            (_, Some(ability @ Action::Ability(_))) => CapabilityKey::Action(ability),
            _ => return,
        };
        let cooldown = capability_profiles[finite_part].map_or(0.0, |profile| profile.cooldown);
        self.start_cooldown(finite_part, now + cooldown);
    }

    fn release_behavior_from_slots(&mut self, behavior: usize) {
        for slot in [&mut self.motion_slot, &mut self.action_slot] {
            if slot.is_some_and(|running| running.behavior == behavior) {
                *slot = None;
            }
        }
    }

    /// Ends the committed behavior stamped `behavior_run_id`; `false` for a stale finish.
    fn finish_committed_behavior(&mut self, behavior_run_id: u32) -> bool {
        let Some(finished) = [self.motion_slot, self.action_slot]
            .into_iter()
            .flatten()
            .find(|running| running.committed && running.behavior_run_id == behavior_run_id)
        else {
            return false;
        };
        self.release_behavior_from_slots(finished.behavior);
        true
    }
}

/// A brain's random stream, seeded from [`BrainRngRootSeed`] and the entity. Kept apart
/// from [`Brain`] so steering gets randomness without the rest of the brain.
#[derive(Component, Debug)]
pub struct BrainRng {
    random: ChaCha8Rng,
}

impl Default for BrainRng {
    fn default() -> Self {
        Self {
            random: ChaCha8Rng::seed_from_u64(0),
        }
    }
}

impl BrainRng {
    /// Uniform value in `min..max`, or `min` for an empty/invalid range.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        if max > min {
            self.random.random_range(min..max)
        } else {
            min
        }
    }

    pub fn direction(&mut self) -> Vec2 {
        let angle = self.random.random_range(0.0..std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }
}

fn seed_brain_rng_and_schedule_first_decision(mut world: DeferredWorld, context: HookContext) {
    let root_seed = world
        .get_resource::<BrainRngRootSeed>()
        .map_or(0, |seed| seed.0);
    let now = world
        .get_resource::<Time<Virtual>>()
        .map_or(0.0, Time::elapsed_secs);
    let entity_seed = context.entity.to_bits().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let Some(mut rng) = world.get_mut::<BrainRng>(context.entity) else {
        return;
    };
    rng.random = ChaCha8Rng::seed_from_u64(root_seed ^ entity_seed);
    let phase = rng.range(0.0, DECISION_PERIOD);
    if let Some(mut brain) = world.get_mut::<Brain>(context.entity) {
        brain.next_decision_at = now + phase;
    }
}

/// An equipment or ability use ended. It ends the behavior stamped `behavior_run_id`
/// only while that behavior still runs, so a stale finish clears nothing.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    pub entity: Entity,
    pub action: Action,
    pub behavior_run_id: u32,
    pub outcome: ActionOutcome,
}

/// How an action reported in [`ActionFinished`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionOutcome {
    Completed,
    /// A committed use its applier could not start. A continuously scored use waits for
    /// its chance instead and never reports this.
    NotStarted,
}

/// Debug readout of a brain's last decision: one explanation per behavior, in
/// definition order. Only brains carrying it pay for the explanations.
#[derive(Component, Debug, Default)]
pub struct ScoreTrace {
    pub behavior_explanations: Vec<BehaviorScoreExplanation>,
}

/// Seconds between a brain's scheduled decisions.
const DECISION_PERIOD: f32 = 0.1;

/// Root seed used to seed each brain's independent random stream.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct BrainRngRootSeed(pub u64);

pub(super) fn plugin(app: &mut App) {
    for &kind in EnemyKind::ALL {
        if let Err(error) = validate_brain(kind.brain()) {
            panic!("{kind:?} brain is invalid: {error}");
        }
    }
    if let Err(error) = validate_brain(&VILLAGER) {
        panic!("villager brain is invalid: {error}");
    }

    app.add_plugins((perception::plugin, action::plugin, motion::plugin))
        .init_resource::<BrainRngRootSeed>()
        .add_observer(release_finished_motion)
        .add_observer(finish_action)
        .add_systems(Startup, seed_brain_rng_root_from_map)
        .add_systems(
            Update,
            decide_behavior
                .after(crate::combat::mana::regenerate_mana)
                .after(tick_equippable_cooldown)
                .in_set(AiSet::Decision),
        );
}

fn seed_brain_rng_root_from_map(
    map_rng: Option<Res<MapRng>>,
    mut root_seed: ResMut<BrainRngRootSeed>,
) {
    if let Some(map_rng) = map_rng {
        root_seed.0 = map_rng.root_seed();
    } else {
        warn!("MapRng is missing; seeding AI randomness with 0");
    }
}
