//! ECS glue for the pure utility brains.
//!
//! Data flows one way around the loop, and every piece of data has one writer:
//! 1. [`senses`] write a [`DecisionContext`] from world state and from the
//!    capability components the entity carries.
//! 2. [`decide`] scores it with the pure [`scoring`] math and writes only
//!    intents: the [`Brain`]'s locomotion mode and maneuver, [`SlotIntent`]
//!    and [`AbilityIntent`].
//! 3. Executors read those intents. [`motion`] steers the body and reports a
//!    [`ManeuverFinished`] message; [`execute`] and the `ability` module apply
//!    equipment and ability use.
//!
//! The brains themselves never see an `Entity`: [`model`] holds their data,
//! [`scoring`] holds the math, [`considerations`] holds what they can sense,
//! and [`brains`] holds the shipped definitions.
//!
//! To add a locomotion behavior, touch these in order:
//! 1. [`model`]: a `MotionMode` variant only for a new steering contract, plus
//!    its one row in `MotionMode::traits` (group, target need).
//! 2. [`motion`]: a behavior file with its component (tuning as pub fields,
//!    memory as private fields) and a steering function.
//! 3. [`model::DecisionContext`] sense fields (or [`model::SelfState`] for
//!    body state) plus [`considerations`] readers.
//! 4. [`brains`]: static options over the mode.
//! 5. An arm in `motion::steer_locomotion`.
//!
//! A committed move is a [`Maneuver`] instead: a variant, a capability
//! component in [`motion`] that implements `ManeuverExecutor` and is listed in
//! `motion::Maneuvers`, and an option on the definition's maneuver channel.
//! A new ability is the same shape: an [`Ability`] variant, a capability
//! component in `ability` that implements `AbilityCapability` and is listed in
//! `AbilityCapabilities`, an applier, and options on the definition's ability
//! channel.

mod brains;
mod considerations;
mod decide;
mod execute;
mod model;
mod motion;
mod scoring;
mod senses;

pub use brains::{BAT, FIRE_MAGE, ICE_MAGE, VILLAGER, WARRIOR};
pub use model::{
    Ability, AbilityTuning, BrainDefinition, CAST_MIN_RANGE, Maneuver, MotionMode,
};
pub use motion::{CastPlant, Dash, Strafe, Swoop, TurnRate, Wander};

use self::decide::{decide_behavior, end_finished_maneuvers};
use self::execute::apply_slot_requests;
use self::model::*;
use self::motion::{ManeuverFinished, drive_motion};
use self::scoring::*;
use self::senses::{
    Perception, sense_calm_and_attack, sense_equipment_and_anchor, sense_self, sense_target,
    sense_vitals,
};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{EquipmentSlot, Mana, apply_slowed, tick_equippable_cooldown},
    world::MapRng,
};

use super::AiSet;

/// A maneuver the brain started, stamped with a fresh epoch so an executor can
/// tell a new run of the same maneuver from the one it just finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveManeuver {
    maneuver: Maneuver,
    epoch: u32,
}

/// A character's utility brain: which static definition it scores against, the
/// locomotion mode and maneuver it is driving, and its decision timers.
///
/// The definition never changes after spawn; decisions rewrite the rest at
/// 10 Hz. Only the decision tick writes it: executors get a read-only view and
/// answer with [`ManeuverFinished`]. Randomness lives apart, in [`BrainRng`].
#[derive(Component, Debug)]
#[require(Perception, AbilityIntent, BrainRng)]
pub struct Brain {
    definition: &'static BrainDefinition,
    mode: MotionMode,
    committed_until: f32,
    dwell_until: f32,
    calm_phase: Option<MotionMode>,
    active: Option<ActiveManeuver>,
    next_epoch: u32,
    maneuver_ready_at: [f32; Maneuver::ALL.len()],
}

impl Brain {
    /// Starts `definition` idle.
    pub fn new(definition: &'static BrainDefinition) -> Self {
        Self {
            definition,
            mode: MotionMode::Idle,
            committed_until: 0.0,
            dwell_until: 0.0,
            calm_phase: None,
            active: None,
            next_epoch: 0,
            maneuver_ready_at: [0.0; Maneuver::ALL.len()],
        }
    }

    /// The static definition this brain scores against.
    pub fn definition(&self) -> &'static BrainDefinition {
        self.definition
    }

    /// The locomotion mode currently driving the body.
    pub fn mode(&self) -> MotionMode {
        self.mode
    }

    /// The maneuver currently holding exclusive control of steering.
    pub fn maneuver(&self) -> Option<Maneuver> {
        self.active.map(|active| active.maneuver)
    }

    fn active_maneuver(&self) -> Option<ActiveManeuver> {
        self.active
    }

    fn maneuver_ready(&self, maneuver: Maneuver, now: f32) -> bool {
        now >= self.maneuver_ready_at[maneuver as usize]
    }

    fn start_maneuver(&mut self, maneuver: Maneuver) {
        self.next_epoch = self.next_epoch.wrapping_add(1);
        self.active = Some(ActiveManeuver {
            maneuver,
            epoch: self.next_epoch,
        });
    }

    /// Drops the running maneuver without a cooldown, because it never ran to
    /// its end (the target vanished).
    fn cancel_maneuver(&mut self) {
        self.active = None;
    }

    /// Releases `maneuver` if it is the running one and holds off the next
    /// attempt until `ready_at`.
    fn finish_maneuver(&mut self, maneuver: Maneuver, ready_at: f32) {
        if self.maneuver() == Some(maneuver) {
            self.active = None;
            self.defer_maneuver(maneuver, ready_at);
        }
    }

    fn defer_maneuver(&mut self, maneuver: Maneuver, ready_at: f32) {
        self.maneuver_ready_at[maneuver as usize] = ready_at;
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
/// state, so it has its own writer instead of living in a sense system.
fn update_mana_recovery(mut casters: Query<(&mut ManaRecovery, Option<&Mana>)>) {
    for (mut recovery, mana) in &mut casters {
        recovery.update(mana.and_then(Mana::fraction).unwrap_or(1.0));
    }
}

/// Per-tick equipment intent, written by `decide_behavior` when present.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SlotIntent([bool; EquipmentSlot::ALL.len()]);

impl SlotIntent {
    pub fn get(&self, slot: EquipmentSlot) -> bool {
        self.0[slot as usize]
    }

    pub fn set_all(&mut self, intents: [bool; EquipmentSlot::ALL.len()]) {
        self.0 = intents;
    }
}

/// Per-tick ability intent, written by `decide_behavior`. Appliers in the
/// `ability` module read it and check the live world before acting.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct AbilityIntent([bool; Ability::ALL.len()]);

impl AbilityIntent {
    /// Whether the brain wants `ability` used this tick.
    pub fn get(&self, ability: Ability) -> bool {
        self.0[ability as usize]
    }

    fn set_all(&mut self, intents: [bool; Ability::ALL.len()]) {
        self.0 = intents;
    }
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

/// Rejects gated options on a brain without a distance band: `pass_gate` would
/// otherwise treat every gate as open and silently ignore the eligibility rule.
fn validate_band_usage(definition: &BrainDefinition) -> Result<(), String> {
    let gated = |options: &[ScoredOption<_>]| options.iter().any(|option| !option.gates.is_empty());
    let any_gated = definition
        .motion_options
        .iter()
        .any(|option| !option.gates.is_empty())
        || definition
            .maneuver_options
            .iter()
            .any(|option| !option.gates.is_empty())
        || definition.slot_options.iter().any(|options| gated(options))
        || definition
            .ability_options
            .iter()
            .any(|options| gated(options));
    if any_gated && definition.band.is_none() {
        return Err("gated options require a distance band".to_string());
    }
    Ok(())
}

pub(super) fn plugin(app: &mut App) {
    for (name, definition) in brains::all() {
        if let Err(error) = validate_brain(definition) {
            panic!("{name} brain is invalid: {error}");
        }
        if let Err(error) = validate_band_usage(definition) {
            panic!("{name} brain is invalid: {error}");
        }
    }

    app.init_resource::<DecisionClock>()
        .init_resource::<AiSeed>()
        .add_message::<ManeuverFinished>()
        .add_systems(Startup, seed_ai_seed)
        .add_systems(
            Update,
            (tick_decision_clock, end_finished_maneuvers)
                .chain()
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            (
                sense_target,
                sense_vitals,
                sense_equipment_and_anchor,
                sense_calm_and_attack,
                update_mana_recovery,
                sense_self,
            )
                .chain()
                .run_if(decision_tick_ready)
                .after(end_finished_maneuvers)
                .after(crate::combat::mana::regenerate_mana)
                .after(tick_equippable_cooldown)
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            decide_behavior
                .run_if(decision_tick_ready)
                .after(sense_self)
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
            validate_band_usage(definition).unwrap_or_else(|error| panic!("{name}: {error}"));
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
        brain.mode = MotionMode::Strafe;
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
        world.run_system_once(sense_target).expect("target sensed");
        world.run_system_once(sense_vitals).expect("vitals sensed");
        world
            .run_system_once(sense_equipment_and_anchor)
            .expect("equipment sensed");
        world
            .run_system_once(sense_calm_and_attack)
            .expect("calm sensed");
        world
            .run_system_once(update_mana_recovery)
            .expect("mana break updated");
        world.run_system_once(sense_self).expect("self sensed");
        world.run_system_once(decide_behavior).expect("decisions run");
    }

    /// The full loop holds: a ready, scored dive starts on the first tick and
    /// leaves the locomotion mode alone.
    #[test]
    fn swoop_lottery_starts_a_dive() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        run_decision_tick(&mut world);
        let brain = world.get::<Brain>(bat).expect("brained bat");
        assert_eq!(brain.maneuver(), Some(Maneuver::Swoop));
        assert_eq!(brain.mode(), MotionMode::Strafe);
    }
}
