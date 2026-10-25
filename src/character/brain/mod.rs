//! ECS glue for the pure utility brains.
//!
//! The vocabulary is standard infinite axis utility:
//! - A [`DecisionContext`] is everything a brain knows this tick.
//! - A [`Consideration`] reads one input from it and maps it to `0.0..=1.0`.
//! - A [`Decision`] pairs an action with a weight and its considerations.
//! - A layer is a list of decisions with a selection rule. The movement layer
//!   is exclusive: the best [`Motion`] wins, and the current continuous motion keeps
//!   momentum. The action layer is independent: every [`Action`] with a
//!   scoring decision fires.
//!
//! Data flows one way around the loop, and every piece of data has one writer:
//! 1. [`senses`] writes a [`DecisionContext`] from world state and from the
//!    capability components the entity carries.
//! 2. [`decide`] runs both layers with the pure [`scoring`] math and writes
//!    only intents: the [`Brain`]'s motion, and [`ActionIntent`].
//! 3. Executors read those intents. [`motion`] steers the body and reports a
//!    [`MotionFinished`] event, which releases the motion and decides again at
//!    once; [`execute`] and the `ability` module apply equipment and ability
//!    use, and signal [`ActionFinished`] when a use ends.
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
//!    [`considerations`] readers.
//! 4. [`brains`]: movement decisions over the motion.
//! 5. An arm in `motion::steer_motion`.
//!
//! A committed move is a finite [`Motion`] instead: a variant, a capability
//! component in [`motion`] that implements `MotionExecutor` and is listed in
//! `motion::FiniteMotions`, and a movement decision over it. A new ability is the
//! same shape: an [`Ability`] variant, a capability component in `ability`
//! that implements `AbilityCapability` and is listed in
//! `AbilityCapabilities`, an applier, and action decisions over it.

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
pub use motion::{CastPlant, Dash, Strafe, Swoop, TurnRate, Wander};

use self::decide::{decide_behavior, release_finished_motion, start_ability_cooldown};
use self::execute::apply_slot_requests;
use self::model::*;
use self::motion::drive_motion;
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

/// A finite motion the brain started, stamped with a fresh epoch so an
/// executor can tell a new run of the same motion from the one it just finished.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveMotion {
    motion: Motion,
    epoch: u32,
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
/// motion it is driving, and its decision timers.
///
/// The definition never changes after spawn; decisions rewrite the rest at
/// 10 Hz. Only the decision tick writes it: executors get a read-only view and
/// answer with [`MotionFinished`]. Randomness lives apart, in [`BrainRng`].
#[derive(Component, Debug)]
#[require(Perception, ActionIntent, BrainRng)]
pub struct Brain {
    definition: &'static BrainDefinition,
    /// The continuous motion underneath any running finite motion: it holds
    /// momentum and steers the frames around one.
    continuous_motion: Motion,
    active: Option<ActiveMotion>,
    next_epoch: u32,
    ready_at: [f32; Cooldown::COUNT],
}

impl Brain {
    /// Starts `definition` calm.
    pub fn new(definition: &'static BrainDefinition) -> Self {
        Self {
            definition,
            continuous_motion: Motion::Calm,
            active: None,
            next_epoch: 0,
            ready_at: [0.0; Cooldown::COUNT],
        }
    }

    /// The static definition this brain scores against.
    pub fn definition(&self) -> &'static BrainDefinition {
        self.definition
    }

    /// The motion currently driving the body: the running finite motion if
    /// there is one, otherwise the continuous motion.
    pub fn motion(&self) -> Motion {
        self.active
            .map_or(self.continuous_motion, |active| active.motion)
    }

    fn active_motion(&self) -> Option<ActiveMotion> {
        self.active
    }

    fn is_ready(&self, cooldown: Cooldown, now: f32) -> bool {
        now >= self.ready_at[cooldown.index()]
    }

    /// Holds `cooldown` off until `ready_at`.
    fn start_cooldown(&mut self, cooldown: Cooldown, ready_at: f32) {
        self.ready_at[cooldown.index()] = ready_at;
    }

    fn start_motion(&mut self, motion: Motion) {
        self.next_epoch = self.next_epoch.wrapping_add(1);
        self.active = Some(ActiveMotion {
            motion,
            epoch: self.next_epoch,
        });
    }

    /// Drops the running finite motion without a cooldown, because it never ran to
    /// its end (the target vanished).
    fn cancel_motion(&mut self) {
        self.active = None;
    }

    /// Releases `motion` if it is the running one and holds off the next
    /// attempt until `ready_at`. Returns whether it was released.
    fn finish_motion(&mut self, motion: Motion, ready_at: f32) -> bool {
        let running = self.active.is_some_and(|active| active.motion == motion);
        if running {
            self.active = None;
            self.start_cooldown(Cooldown::Motion(motion), ready_at);
        }
        running
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

/// Per-tick action intent, written by `decide_behavior`. The equipment applier
/// and the `ability` appliers read it and check the live world before acting.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionIntent([bool; Action::COUNT]);

impl ActionIntent {
    /// Whether the brain wants `action` performed this tick.
    pub fn get(&self, action: Action) -> bool {
        self.0[action.index()]
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
        .add_observer(start_ability_cooldown)
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
        .add_systems(Update, apply_slot_requests.in_set(AiSet::Execute))
        .add_systems(
            Update,
            drive_motion.in_set(AiSet::Motion).after(apply_slowed),
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
    /// the cast band, then advances the clock past one decision tick.
    fn swooping_bat(chance: f32, interval: f32) -> (World, Entity) {
        let mut world = World::new();
        let mut time = Time::<Virtual>::default();
        time.advance_by(std::time::Duration::from_millis(110));
        world.insert_resource(time);
        world.insert_resource(DecisionClock::default());
        world.insert_resource(AiSeed::default());
        let target = world.spawn_empty().id();
        let mut brain = Brain::new(&BAT);
        brain.continuous_motion = Motion::Strafe(CAST_BAND);
        let bat = world
            .spawn((
                brain,
                Targeting(target),
                TargetInfo {
                    distance: CAST_BAND.max() - 50.0,
                    direction: Vec2::X,
                    target_velocity: Vec2::ZERO,
                    line_of_sight: true,
                    in_vision_cone: true,
                },
                Transform::default(),
                Strafe::new(),
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
    /// leaves the continuous motion underneath it alone.
    #[test]
    fn swoop_lottery_starts_a_dive() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        run_decision_tick(&mut world);
        let brain = world.get::<Brain>(bat).expect("brained bat");
        assert_eq!(brain.motion(), Motion::Swoop);
        assert_eq!(brain.continuous_motion, Motion::Strafe(CAST_BAND));
    }
}
