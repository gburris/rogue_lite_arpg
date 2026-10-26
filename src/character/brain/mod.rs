//! ECS glue for the pure utility brains.
//!
//! The vocabulary is standard infinite axis utility:
//! - A [`DecisionContext`] is everything a brain knows this tick.
//! - A [`Consideration`] extracts one raw input from it and maps it to `0.0..=1.0`.
//! - A [`Decision`] pairs a [`Behavior`] with a weight and its considerations.
//!   A behavior is a [`Motion`] and/or an [`Action`] plus a [`CommitPolicy`].
//! - A brain holds at most one motion and one action at a time. Behaviors take
//!   those two slots greedily from the highest score down; a holder keeps
//!   momentum, and a locked holder is not re-scored at all.
//!
//! Data flows one way around the loop, and every piece of data has one writer:
//! 1. [`senses`] writes a [`DecisionContext`] from world state and from the
//!    capability components the entity carries.
//! 2. [`decide`] runs the pure [`scoring`] math and writes only which
//!    behavior holds each slot on the [`Brain`].
//! 3. Executors read the held motion and action. [`motion`] steers the body and
//!    reports a [`MotionFinished`] event, which releases the motion and
//!    decides again at once; [`execute`] and the `ability` module apply
//!    equipment and ability use, and signal [`ActionFinished`] when a use ends.
//!
//! The brains themselves never see an `Entity`: [`model`] holds their data,
//! [`scoring`] holds the math, [`considerations`] holds what they can sense,
//! and [`brains`] holds the shipped definitions.
//!
//! To add a continuous motion, touch these in order:
//! 1. [`model`]: a `Motion` variant only for a new steering contract.
//! 2. [`motion`]: a behavior file with its component (tuning as pub fields,
//!    memory as private fields) and a steering function.
//! 3. [`model::DecisionContext`] fields filled in [`senses`], plus
//!    [`considerations`] input extractors.
//! 4. [`brains`]: behaviors over the motion.
//! 5. An arm in `motion::steer_continuous_motion`.
//!
//! A committed move is a finite [`Motion`] instead: a variant, a capability
//! component in [`motion`] that implements `MotionExecutor` and is listed in
//! `motion::FiniteMotions`, and a locked behavior over it. A new ability is the
//! same shape: an [`Ability`] variant, a capability component in `ability`
//! that implements `AbilityCapability` and is listed in
//! `AbilityCapabilities`, an applier, and action behaviors over it.

mod ability;
mod brains;
mod considerations;
mod decide;
mod execute;
mod model;
mod motion;
mod scoring;
mod senses;

pub use ability::{Bite, Breath, Heal};
pub use brains::{BAT, FIRE_MAGE, ICE_MAGE, VILLAGER, WARRIOR};
pub use model::{
    Ability, Action, BrainDefinition, CAST_MAX_RANGE, CAST_MIN_RANGE, Interval, Motion, Tuning,
};
pub use motion::{Dash, Orbit, Swoop, TurnRate, Wander};

use self::decide::{
    abort_failed_action, abort_failed_motion, decide_behavior, finish_action,
    release_finished_motion,
};
use self::execute::{StartedLockedEquipmentUse, apply_equipment_use_requests};
use self::model::*;
use self::motion::steer_characters_from_brain_motion;
use self::scoring::*;
use self::senses::{Perception, sense};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{Mana, apply_slowed, tick_equippable_cooldown},
    world::MapRng,
};

use super::AiSet;

/// A locked motion as executors see it, stamped with a fresh epoch so an
/// executor can tell a new run of the same motion from the one it just finished.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveMotion {
    motion: Motion,
    epoch: u32,
}

/// A locked action as the equipment applier sees it, stamped like [`ActiveMotion`].
#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveAction {
    action: Action,
    epoch: u32,
}

/// A behavior that won a slot, and how far its run has progressed. A bundle
/// holds both slots, one record each.
#[derive(Clone, Copy, Debug)]
struct RunningBehavior {
    /// Index into the definition; the identity momentum is keyed on.
    behavior: usize,
    epoch: u32,
    locked: bool,
    started_at: f32,
    /// When line of sight to the target was first lost, while it stays lost.
    los_lost_since: Option<f32>,
    /// This slot's finite part reported done.
    finished: bool,
}

/// Something a brain spaces out: a finite motion or an ability.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cooldown {
    Motion(Motion),
    Ability(Ability),
}

impl Cooldown {
    const COUNT: usize = Motion::COUNT + Ability::ALL.len();

    const fn index(self) -> usize {
        match self {
            Self::Motion(motion) => motion.index(),
            Self::Ability(ability) => Motion::COUNT + ability as usize,
        }
    }
}

/// A character's utility brain: which static definition it scores against, the
/// behaviors holding its motion and action slots, and its cooldowns.
///
/// The definition never changes after spawn; decisions rewrite the rest at
/// 10 Hz. Only the decision side writes it: executors get a read-only view and
/// answer with [`MotionFinished`], [`MotionAborted`], [`ActionFinished`] or
/// [`ActionAborted`]. Randomness lives apart, in [`BrainRng`].
#[derive(Component, Debug)]
#[require(Perception, BrainRng, StartedLockedEquipmentUse)]
pub struct Brain {
    definition: &'static BrainDefinition,
    motion: Option<RunningBehavior>,
    action: Option<RunningBehavior>,
    next_epoch: u32,
    ready_at: [f32; Cooldown::COUNT],
}

impl Brain {
    /// Starts `definition` with both slots empty.
    pub fn new(definition: &'static BrainDefinition) -> Self {
        Self {
            definition,
            motion: None,
            action: None,
            next_epoch: 0,
            ready_at: [0.0; Cooldown::COUNT],
        }
    }

    /// The static definition this brain scores against.
    pub fn definition(&self) -> &'static BrainDefinition {
        self.definition
    }

    /// The motion currently driving the body: the motion slot's holder, or
    /// `None` while the slot is empty and the body stands still.
    pub fn motion(&self) -> Option<Motion> {
        self.motion
            .and_then(|running| self.behavior(running).motion)
    }

    /// The action the brain currently wants performed, if any.
    pub fn action(&self) -> Option<Action> {
        self.action
            .and_then(|running| self.behavior(running).action)
    }

    fn behavior(&self, running: RunningBehavior) -> Behavior {
        self.definition.behaviors[running.behavior].value
    }

    /// The locked motion an executor is carrying out, if any.
    fn active_motion(&self) -> Option<ActiveMotion> {
        let running = self.motion.filter(|running| running.locked)?;
        Some(ActiveMotion {
            motion: self.behavior(running).motion?,
            epoch: running.epoch,
        })
    }

    /// The locked action an applier is carrying out, if any.
    fn active_action(&self) -> Option<ActiveAction> {
        let running = self.action.filter(|running| running.locked)?;
        Some(ActiveAction {
            action: self.behavior(running).action?,
            epoch: running.epoch,
        })
    }

    /// A run holding a lock. A bundle holds both slots, so either record serves.
    fn locked_running_behavior_mut(&mut self) -> Option<&mut RunningBehavior> {
        self.motion
            .as_mut()
            .filter(|running| running.locked)
            .or(self.action.as_mut().filter(|running| running.locked))
    }

    fn is_cooldown_ready(&self, cooldown: Cooldown, now: f32) -> bool {
        now >= self.ready_at[cooldown.index()]
    }

    /// Holds `cooldown` off until `ready_at`.
    fn start_cooldown(&mut self, cooldown: Cooldown, ready_at: f32) {
        self.ready_at[cooldown.index()] = ready_at;
    }

    fn holders(&self) -> SlotHolders {
        let holder = |running: RunningBehavior| SlotHolder {
            behavior: running.behavior,
            locked: running.locked,
        };
        SlotHolders {
            motion: self.motion.map(holder),
            action: self.action.map(holder),
        }
    }

    /// Hands each slot to the behavior that won it. A slot nothing claimed
    /// empties unless a lock holds it.
    fn assign_slots(&mut self, assignment: Assignment, now: f32) {
        match assignment.motion {
            Some(index) => self.motion = self.run(self.motion, index, now),
            None if self.motion.is_none_or(|running| !running.locked) => self.motion = None,
            None => {}
        }
        match assignment.action {
            Some(index) => self.action = self.run(self.action, index, now),
            None if self.action.is_none_or(|running| !running.locked) => self.action = None,
            None => {}
        }
    }

    /// The run of behavior `index`: the current one if it already holds the
    /// slot, otherwise a fresh one stamped with a new epoch.
    fn run(
        &mut self,
        current: Option<RunningBehavior>,
        index: usize,
        now: f32,
    ) -> Option<RunningBehavior> {
        if current.is_some_and(|running| running.behavior == index) {
            return current;
        }
        self.next_epoch = self.next_epoch.wrapping_add(1);
        Some(RunningBehavior {
            behavior: index,
            epoch: self.next_epoch,
            locked: self.definition.behaviors[index].value.commit.is_lock(),
            started_at: now,
            los_lost_since: None,
            finished: false,
        })
    }

    /// Drops the locked behavior from every slot it holds, without a cooldown,
    /// because it never ran to its end (an abort).
    fn abort_locked_behavior(&mut self) {
        let locked = [self.motion, self.action]
            .into_iter()
            .flatten()
            .find(|running| running.locked);
        if let Some(running) = locked {
            self.release(running.behavior);
        }
    }

    fn release(&mut self, behavior: usize) {
        for slot in [&mut self.motion, &mut self.action] {
            if slot.is_some_and(|running| running.behavior == behavior) {
                *slot = None;
            }
        }
    }

    /// Marks the locked motion's part done if `motion` is it. Returns `None`
    /// when it is not, otherwise whether that completed the lock and released it.
    fn finish_motion_part(&mut self, motion: Motion, now: f32) -> Option<bool> {
        let definition = self.definition;
        let running = self.motion.as_mut().filter(|running| running.locked)?;
        if definition.behaviors[running.behavior].value.motion != Some(motion) {
            return None;
        }
        running.finished = true;
        Some(self.release_if_complete(now))
    }

    /// Marks the locked action's part done if `action` is it (`None` stands for
    /// any equipment use). Returns `None` when it is not, otherwise whether
    /// that completed the lock and released it.
    fn finish_action_part(&mut self, action: Option<Action>, now: f32) -> Option<bool> {
        let definition = self.definition;
        let running = self.action.as_mut().filter(|running| running.locked)?;
        let held = definition.behaviors[running.behavior].value.action?;
        let matches = match action {
            Some(finished) => held == finished,
            None => matches!(held, Action::Use(_)),
        };
        if !matches {
            return None;
        }
        running.finished = true;
        Some(self.release_if_complete(now))
    }

    /// Releases the locked behavior once every finite part it holds is done and
    /// its minimum hold has passed. A lock that finished early is released by a
    /// later call, which the decision tick makes.
    fn release_if_complete(&mut self, now: f32) -> bool {
        let Some(locked) = [self.motion, self.action]
            .into_iter()
            .flatten()
            .find(|running| running.locked)
        else {
            return false;
        };
        let behavior = self.behavior(locked);
        let finished = |slot: Option<RunningBehavior>| {
            slot.is_some_and(|running| running.behavior == locked.behavior && running.finished)
        };
        let motion_done = !behavior.motion.is_some_and(Motion::is_finite) || finished(self.motion);
        let action_done = behavior.action.is_none() || finished(self.action);
        let released =
            motion_done && action_done && now - locked.started_at >= behavior.commit.min_seconds();
        if released {
            self.release(locked.behavior);
        }
        released
    }
}

/// A brain's private random stream, seeded from [`AiSeed`] and the entity when
/// it is added. Executors draw from it every frame and the decision tick draws
/// from it too, so keeping it apart gives steering code randomness without
/// handing it the rest of the brain.
#[derive(Component, Debug)]
#[component(on_add = seed_brain_rng)]
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

    /// A uniformly random unit direction.
    pub fn direction(&mut self) -> Vec2 {
        let angle = self.random.random_range(0.0..std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }
}

fn seed_brain_rng(mut world: DeferredWorld, context: HookContext) {
    let root_seed = world.get_resource::<AiSeed>().map_or(0, |seed| seed.0);
    let entity_seed = context.entity.to_bits().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    if let Some(mut rng) = world.get_mut::<BrainRng>(context.entity) {
        rng.random = ChaCha8Rng::seed_from_u64(root_seed ^ entity_seed);
    }
}

/// Held mana-break state for casters. A single threshold would flicker on
/// and off while regeneration crosses it, so the break enters at `trigger_at`
/// and holds until the pool reaches `resume_at`.
#[derive(Component, Clone, Copy, Debug)]
pub struct ManaRecovery {
    pub trigger_at: f32,
    pub resume_at: f32,
    recovering: bool,
}

impl ManaRecovery {
    /// Enters a mana break at `trigger_at` and leaves it at `resume_at`.
    pub fn new(trigger_at: f32, resume_at: f32) -> Self {
        assert!(
            trigger_at.is_finite()
                && resume_at.is_finite()
                && trigger_at >= 0.0
                && trigger_at < resume_at
                && resume_at <= 1.0,
            "mana break needs 0 <= trigger_at < resume_at <= 1"
        );
        Self {
            trigger_at,
            resume_at,
            recovering: false,
        }
    }

    /// Whether the caster is running out a mana break.
    fn recovering(&self) -> bool {
        self.recovering
    }

    /// Advances the break from the current mana fraction.
    fn update(&mut self, mana_fraction: f32) {
        let mana_fraction = mana_fraction.clamp(0.0, 1.0);
        if self.recovering {
            if mana_fraction >= self.resume_at {
                self.recovering = false;
            }
        } else if mana_fraction <= self.trigger_at {
            self.recovering = true;
        }
    }
}

/// Advances every caster's mana break on the decision tick. The break is body
/// state, so it has its own writer instead of living in the sense system.
fn update_mana_recovery(mut casters: Query<(&mut ManaRecovery, Option<&Mana>)>) {
    for (mut recovery, mana) in &mut casters {
        recovery.update(mana.and_then(Mana::fraction).unwrap_or(1.0));
    }
}

/// An equipment or ability use ran to its end. A character holds at most one
/// action at a time, so `action` is only needed to start an ability's
/// cooldown; equipment keeps its own cooldown and sends `None`.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    pub entity: Entity,
    pub action: Option<Action>,
}

/// A locked action could not start. The brain drops the lock without a
/// cooldown and decides again on the spot.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionAborted {
    pub entity: Entity,
}

/// Shared ~10 Hz decision clock.
#[derive(Resource)]
pub struct DecisionClock(pub Timer);

impl Default for DecisionClock {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

/// Root seed used to seed each brain's independent random stream.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct AiSeed(pub u64);

pub(super) fn plugin(app: &mut App) {
    for (name, definition) in brains::all() {
        if let Err(error) = validate_brain(definition) {
            panic!("{name} brain is invalid: {error}");
        }
    }

    app.add_plugins(ability::plugin)
        .init_resource::<DecisionClock>()
        .init_resource::<AiSeed>()
        .add_observer(release_finished_motion)
        .add_observer(abort_failed_motion)
        .add_observer(finish_action)
        .add_observer(abort_failed_action)
        .add_systems(Startup, seed_ai_seed)
        .add_systems(Update, tick_decision_clock.in_set(AiSet::Decision))
        .add_systems(
            Update,
            (update_mana_recovery, sense)
                .chain()
                .run_if(decision_tick_ready)
                .after(tick_decision_clock)
                .after(crate::combat::mana::regenerate_mana)
                .after(tick_equippable_cooldown)
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            decide_behavior
                .run_if(decision_tick_ready)
                .after(sense)
                .in_set(AiSet::Decision),
        )
        .add_systems(Update, apply_equipment_use_requests.in_set(AiSet::Execute))
        .add_systems(
            Update,
            steer_characters_from_brain_motion
                .in_set(AiSet::Motion)
                .after(apply_slowed),
        );
}

fn seed_ai_seed(map_rng: Option<Res<MapRng>>, mut ai_seed: ResMut<AiSeed>) {
    if let Some(map_rng) = map_rng {
        ai_seed.0 = map_rng.root_seed();
    } else {
        warn!("MapRng is missing; seeding AI randomness with 0");
    }
}

fn tick_decision_clock(time: Res<Time<Virtual>>, mut clock: ResMut<DecisionClock>) {
    clock.0.tick(time.delta());
}

fn decision_tick_ready(clock: Res<DecisionClock>) -> bool {
    clock.0.just_finished()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::vision::{TargetInfo, Targeting};
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn every_brain_is_valid() {
        for (name, definition) in brains::all() {
            validate_brain(definition).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
    }

    /// Spawns a circling bat with a guaranteed lottery and a live target in
    /// the cast distance range, then advances the clock past one decision tick.
    fn swooping_bat(chance: f32, interval: f32) -> (World, Entity) {
        let mut world = World::new();
        let mut time = Time::<Virtual>::default();
        time.advance_by(std::time::Duration::from_millis(110));
        world.insert_resource(time);
        world.insert_resource(DecisionClock::default());
        world.insert_resource(AiSeed::default());
        let target = world.spawn_empty().id();
        let mut brain = Brain::new(&BAT);
        let orbit = BAT
            .behaviors
            .iter()
            .position(|decision| decision.value.motion == Some(Motion::Orbit(CAST_DISTANCE_RANGE)))
            .expect("the bat lists its orbit");
        brain.assign_slots(
            Assignment {
                motion: Some(orbit),
                action: None,
            },
            0.0,
        );
        let bat = world
            .spawn((
                brain,
                Targeting(target),
                TargetInfo {
                    distance: CAST_DISTANCE_RANGE.max() - 50.0,
                    direction: Vec2::X,
                    target_velocity: Vec2::ZERO,
                    line_of_sight: true,
                    in_vision_cone: true,
                },
                Transform::default(),
                Orbit::new(),
                Swoop::new(chance, interval),
            ))
            .id();
        (world, bat)
    }

    fn run_decision_tick(world: &mut World) {
        world
            .run_system_once(update_mana_recovery)
            .expect("mana break updated");
        world.run_system_once(sense).expect("brain sensed");
        world
            .run_system_once(decide_behavior)
            .expect("decisions run");
    }

    /// The full loop holds: a ready, scored dive starts on the first tick and
    /// locks the motion slot.
    #[test]
    fn swoop_lottery_starts_a_dive() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        run_decision_tick(&mut world);
        let brain = world.get::<Brain>(bat).expect("brained bat");
        assert_eq!(brain.motion(), Some(Motion::Swoop));
        assert!(brain.active_motion().is_some());
    }
}
