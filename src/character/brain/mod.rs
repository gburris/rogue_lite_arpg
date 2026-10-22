//! ECS glue for the pure utility brains.
//!
//! This module owns perceived state, the shared decision clock, and the
//! translation between a [`BrainDecision`] and equipment/motion components.
//! The brains themselves never see an `Entity`: [`model`] holds their data,
//! [`scoring`] holds the math, [`considerations`] holds what they can sense,
//! To add a motion behavior, touch these in order:
//! 1. [`model`]: a `MotionMode` variant only for a new steering contract. A
//!    one-shot commitment also goes in [`model::MotionMode::is_commitment`];
//!    target need and the arbiter bypass follow from it.
//! 2. A behavior component: tuning as pub fields, memory as private fields.
//! 3. [`model::DecisionContext`] sense fields plus [`considerations`] readers.
//! 4. [`brains`]: static options over the mode.
//! 5. `MotionMode::group`, a `drive_motion` match arm, and a steering helper.
//! 6. An `AiSet::Execute` applier for damage or consumed intents.

mod brains;
mod considerations;
mod model;
mod scoring;

pub use brains::{BAT, FIRE_MAGE, ICE_MAGE, VILLAGER, WARRIOR};
pub use model::*;
pub use scoring::*;
use brains::MELEE_HYSTERESIS;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{
        AttackState, Consumable, Consume, EquipmentRead, EquipmentSlot, EquipmentUsage, Health,
        Items, Mana, TILE_SIZE, apply_slowed, tick_equippable_cooldown,
    },
    world::MapRng,
};

use super::{
    AiSet,
    anchor::Anchor,
    state::SimpleMotion,
    vision::{TargetInfo, Targeting},
};

const ANCHOR_ARRIVAL_RADIUS: f32 = TILE_SIZE as f32 / 2.0;

/// Wander speed as a fraction of max speed; ambling keeps steps inside small anchors.
const WANDER_SPEED_SCALE: f32 = 0.4;
const STRAFE_REROLL_MIN: f32 = 1.5;
const STRAFE_REROLL_MAX: f32 = 2.5;
const STRAFE_SWITCH_CHANCE: f32 = 0.5;
const STRAFE_RADIAL_GAIN: f32 = 0.5;
const SMART_STRAFE_REROLL_MIN: f32 = 0.5;
const SMART_STRAFE_REROLL_MAX: f32 = 0.9;
const SMART_STRAFE_INPUT_DEADBAND: f32 = 20.0;
const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Half-angle of a warrior's break cone around straight away from its target.
const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

/// Half-angle of the fire mage's step-in spread around the target bearing.
const BURST_STEP_IN_SPREAD: f32 = 0.35;
/// Small outward bias on a tangent burst so a volley does not walk the mage
/// into the target when the band is already tight.
const BURST_OUTWARD_BIAS: f32 = 0.15;

/// Selects one of the built-in static brains. Locomotion memory and per-tick
/// intents are opt-in at spawn; consumers query them as `Option`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
#[require(MotionState, BrainRandom)]
pub enum BrainKind {
    Warrior,
    FireMage,
    IceMage,
    Villager,
    /// Flying enemy: circles its target and breathes fire without a weapon.
    Bat,
}

impl BrainKind {
    /// Every shipped brain, in the order they are validated.
    pub const ALL: &[Self] = &[
        Self::Warrior,
        Self::FireMage,
        Self::IceMage,
        Self::Villager,
        Self::Bat,
    ];

    /// The static definition this kind scores against.
    pub(crate) fn definition(self) -> &'static Brain {
        match self {
            Self::Warrior => &WARRIOR,
            Self::FireMage => &FIRE_MAGE,
            Self::IceMage => &ICE_MAGE,
            Self::Villager => &VILLAGER,
            Self::Bat => &BAT,
        }
    }

    /// Distance band this kind's gates measure against.
    pub(crate) fn band(self) -> Option<DistanceBand> {
        match self {
            Self::Warrior => Some(MELEE_BAND),
            Self::FireMage | Self::IceMage | Self::Bat => Some(CAST_BAND),
            Self::Villager => None,
        }
    }
}

/// Maximum turn rate in radians per second. Absent means snap.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// The motion mode currently being driven, plus its commitment window.
#[derive(Component, Debug, Default)]
pub struct MotionState {
    pub mode: MotionMode,
    pub committed_until: f32,
    pub dwell_until: f32,
    calm_phase: Option<MotionMode>,
}

/// How one entity wanders. Opt-in at spawn; absent means that entity never wanders.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    pub min: f32,
    pub max: f32,
    pub speed_scale: f32,
    direction: Vec2,
    next_direction_at: f32,
}
impl Wander {
    /// Holds each direction for a uniform roll in `min..=max` seconds.
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            speed_scale: WANDER_SPEED_SCALE,
            direction: Vec2::ZERO,
            next_direction_at: 0.0,
        }
    }
}
/// How one entity strafes. Opt-in at spawn; absent means never strafes.
#[derive(Component, Clone, Copy, Debug)]
pub struct Strafe {
    pub band: DistanceBand,
    pub reroll_min: f32,
    pub reroll_max: f32,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub standoff_range: f32,
    /// Whether the orbit side should follow the target's tangential movement.
    pub smart_orbit: bool,
    standoff: f32,
    orbit: Option<Orbit>,
    reroll_at: f32,
}
impl Strafe {
    /// Orbits `band` with the default rhythm and no radius standoff.
    pub fn new(band: DistanceBand) -> Self {
        Self {
            band,
            reroll_min: STRAFE_REROLL_MIN,
            reroll_max: STRAFE_REROLL_MAX,
            switch_chance: STRAFE_SWITCH_CHANCE,
            radial_gain: STRAFE_RADIAL_GAIN,
            standoff_range: 0.0,
            smart_orbit: false,
            standoff: 0.0,
            orbit: None,
            reroll_at: 0.0,
        }
    }

    /// Orbits with a short reaction window and counter-moves the target's
    /// tangential motion instead of rerolling the side at random.
    pub fn smart(band: DistanceBand) -> Self {
        Self {
            reroll_min: SMART_STRAFE_REROLL_MIN,
            reroll_max: SMART_STRAFE_REROLL_MAX,
            switch_chance: 0.0,
            smart_orbit: true,
            ..Self::new(band)
        }
    }

    /// Also rerolls a radius offset in `-standoff_range..=standoff_range`.
    pub fn with_standoff(band: DistanceBand, standoff_range: f32) -> Self {
        Self {
            standoff_range,
            ..Self::new(band)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Orbit {
    #[default]
    Clockwise,
    CounterClockwise,
}

impl Orbit {
    fn flipped(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }

    fn sign(self) -> f32 {
        match self {
            Self::Clockwise => -1.0,
            Self::CounterClockwise => 1.0,
        }
    }
}
/// How one biter swoops. Opt-in at spawn; absent means never swoops.
///
/// The roll interval restarts when a dive finishes, so a long pass does not
/// return to a lottery that is already due. The exit also keeps a decaying
/// speed boost for a short recovery window, which lets the biter settle back
/// into its circle instead of snapping from the dive speed to normal.
#[derive(Component, Clone, Debug)]
pub struct Swoop {
    /// Probability of a dive on each roll, `0.0..=1.0`.
    pub chance: f32,
    /// Seconds between rolls while circling or keeping distance. A completed
    /// dive starts the next interval when the dive finishes.
    pub interval: f32,
    pub damage: f32,
    pub speed_boost: f32,
    pub exit_turn: f32,
    next_roll_at: f32,
    called: bool,
    pass: Option<SwoopPass>,
    started_at: f32,
    bitten: bool,
    recovery_until: f32,
}
impl Swoop {
    /// Rolls `chance` every `interval` seconds for `damage` on contact.
    pub fn new(chance: f32, interval: f32, damage: f32) -> Self {
        Self {
            chance,
            interval,
            damage,
            speed_boost: SWOOP_SPEED_BOOST,
            exit_turn: SWOOP_EXIT_TURN,
            next_roll_at: 0.0,
            called: false,
            pass: None,
            started_at: 0.0,
            bitten: false,
            recovery_until: 0.0,
        }
    }
    pub fn is_active(&self) -> bool {
        // The latch alone counts: a pass may never be built when the bearing
        // collapses, and gating on the pass would leave the call with no
        // timeout owner.
        self.called
    }
    /// Ends the dive, starts the post-dive roll interval, and opens the exit
    /// recovery window for dives that actually flew.
    pub(crate) fn finish(&mut self, now: f32) {
        let dove = self.pass.is_some();
        if self.called {
            self.next_roll_at = now + self.interval;
        }
        if dove {
            self.recovery_until = now + SWOOP_RECOVERY_TIME;
        }
        self.called = false;
        self.pass = None;
        self.bitten = false;
    }
    /// Extra speed still in effect while the biter settles out of a dive.
    /// Smoothstep keeps the first frames near full dive speed and lands on
    /// exactly `1.0` at the end of the recovery window.
    pub(crate) fn recovery_boost(&self, now: f32) -> f32 {
        if now >= self.recovery_until {
            return 1.0;
        }
        let remaining = ((self.recovery_until - now) / SWOOP_RECOVERY_TIME).clamp(0.0, 1.0);
        let eased = remaining * remaining * (3.0 - 2.0 * remaining);
        1.0 + self.speed_boost * eased
    }

    pub(crate) fn bite(&mut self) -> bool {
        let available = !self.bitten;
        self.bitten = true;
        available
    }
    pub(crate) fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        self.pass
            .as_ref()
            .is_some_and(|pass| pass.cleared(position, target_distance))
    }
    pub(crate) fn timed_out(&self, now: f32) -> bool {
        now - self.started_at > SWOOP_TIMEOUT
    }
}
const SWOOP_SPEED_BOOST: f32 = 1.0;
const SWOOP_EXIT_TURN: f32 = 2.0 * std::f32::consts::FRAC_PI_3;
const SWOOP_LOOKAHEAD: f32 = 40.0;
const SWOOP_TRACK_LIMIT: f32 = 0.25;
const SWOOP_TRACK_RATIO: f32 = 0.5;
const SWOOP_ARC_SAMPLES: usize = 12;
/// Seconds over which the dive's speed boost fades after the pass ends.
const SWOOP_RECOVERY_TIME: f32 = 0.4;
pub(crate) const SWOOP_TIMEOUT: f32 = 5.0;

/// One dive's committed flight path through a target snapshot and back out.
#[derive(Clone, Copy, Debug)]
struct SwoopPass {
    start: Vec2,
    control: Vec2,
    end: Vec2,
    target: Vec2,
    origin_target: Vec2,
    exit_bearing: Vec2,
    entry_bearing: Vec2,
    exit_radius: f32,
    length: f32,
    progress: f32,
}

impl SwoopPass {
    fn new(start: Vec2, target: Vec2, exit_radius: f32, heading: Vec2, exit_turn: f32) -> Self {
        let entry_bearing = (start - target).normalize_or_zero();
        // A zero bearing means the biter sits on the target: there is no arc
        // to fly, and the pass collapses to a point that ends at once.
        let orbit = entry_bearing.perp().dot(heading);
        let side = if orbit > 0.0 { -1.0 } else { 1.0 };
        let exit_bearing = entry_bearing.rotate(Vec2::from_angle(side * exit_turn));
        let mut pass = Self {
            start,
            control: Vec2::ZERO,
            end: Vec2::ZERO,
            target,
            origin_target: target,
            exit_bearing,
            entry_bearing,
            exit_radius,
            length: 0.0,
            progress: 0.0,
        };
        pass.rebuild();
        pass
    }

    fn rebuild(&mut self) {
        self.end = self.target + self.exit_bearing * self.exit_radius;
        // Placing the control point here puts the arc's midpoint exactly on
        // the aim point, which is what keeps the pass aimed through it.
        self.control = self.target * 2.0 - (self.start + self.end) * 0.5;
        self.length = self.estimate_length();
    }

    fn track(&mut self, live_target: Vec2, max_step: f32) {
        let limit = self.exit_radius * SWOOP_TRACK_LIMIT;
        let desired =
            self.origin_target + (live_target - self.origin_target).clamp_length_max(limit);
        let step = (desired - self.target).clamp_length_max(max_step);
        if step.length_squared() <= f32::EPSILON {
            return;
        }
        self.target += step;
        self.rebuild();
    }

    fn point(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let inverse = 1.0 - t;
        self.start * (inverse * inverse) + self.control * (2.0 * inverse * t) + self.end * (t * t)
    }

    fn estimate_length(&self) -> f32 {
        (1..=SWOOP_ARC_SAMPLES)
            .map(|step| {
                let t0 = (step - 1) as f32 / SWOOP_ARC_SAMPLES as f32;
                let t1 = step as f32 / SWOOP_ARC_SAMPLES as f32;
                self.point(t0).distance(self.point(t1))
            })
            .sum()
    }

    /// Advances the arc by `distance` world units, clamped to its end.
    fn advance(&mut self, distance: f32) {
        if self.length > f32::EPSILON {
            self.progress = (self.progress + distance / self.length).clamp(0.0, 1.0);
        } else {
            self.progress = 1.0;
        }
    }

    /// Point `distance` world units further along the arc from the current
    /// progress, clamped to the end.
    fn lead(&self, distance: f32) -> Vec2 {
        let lead = if self.length > f32::EPSILON {
            self.progress + distance / self.length
        } else {
            1.0
        };
        self.point(lead)
    }

    /// Whether the biter has crossed to the far side of the target snapshot
    /// and opened back out to the exit radius, or flown the whole arc.
    fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        if self.progress >= 1.0 {
            return true;
        }
        let passed = (position - self.target).dot(self.entry_bearing) < 0.0;
        passed && target_distance >= self.exit_radius
    }
}
/// How one warrior breaks away after a swing. Opt-in at spawn; absent means
/// the warrior never kites.
///
/// A break fixes its bearing once, so the warrior runs a straight line out and
/// then turns back instead of curving around its target. The chance is rolled
/// once per completed swing, which keeps repeat swings without a break rare.
#[derive(Component, Clone, Debug)]
pub struct Kite {
    /// Probability of breaking away after any completed swing, `0.0..=1.0`.
    pub chance: f32,
    /// Shortest and longest run of one break, in world units.
    pub min_run: f32,
    pub max_run: f32,
    /// Half-angle of the break cone around straight away from the target.
    pub spread: f32,
    direction: Vec2,
    remaining: f32,
    called: bool,
}

impl Kite {
    /// Breaks away on `chance` of swings for a rolled `min_run..max_run` run.
    pub fn new(chance: f32, min_run: f32, max_run: f32) -> Self {
        Self {
            chance,
            min_run,
            max_run,
            spread: KITE_SPREAD,
            direction: Vec2::ZERO,
            remaining: 0.0,
            called: false,
        }
    }

    /// Whether a break is currently called.
    fn is_called(&self) -> bool {
        self.called
    }

    /// Commits a break along `direction` for `run` world units.
    fn call(&mut self, direction: Vec2, run: f32) {
        self.direction = direction;
        self.remaining = run;
        self.called = true;
    }

    /// Ends the break; the next decision tick hands the warrior back to chase.
    fn finish(&mut self) {
        self.called = false;
        self.remaining = 0.0;
    }

    /// Steers the break while consuming `distance` world units of the run.
    /// Returns `None` once the run is spent.
    fn step(&mut self, distance: f32) -> Option<Vec2> {
        if !self.called {
            return None;
        }
        if distance >= self.remaining {
            self.finish();
            return None;
        }
        self.remaining -= distance;
        Some(self.direction)
    }
}

/// One committed straight run along a bearing fixed at the call.
#[derive(Clone, Copy, Debug, Default)]
struct StraightRun {
    direction: Vec2,
    remaining: f32,
    called: bool,
}

impl StraightRun {
    fn call(&mut self, direction: Vec2, run: f32) {
        self.direction = direction;
        self.remaining = run.max(0.0);
        self.called = true;
    }

    fn finish(&mut self) {
        self.called = false;
        self.remaining = 0.0;
    }

    fn step(&mut self, distance: f32) -> Option<Vec2> {
        if !self.called {
            return None;
        }
        if distance >= self.remaining {
            self.finish();
            return None;
        }
        self.remaining -= distance;
        Some(self.direction)
    }
}

/// Fire mage's after-volley hop. Opt-in at spawn; absent means no burst.
///
/// The cast edge is the same one the warrior's kite uses: a finished volley
/// asks for one tangent step, and the step fixes its bearing so the mage keeps
/// its new firing angle instead of curving back toward the old one.
#[derive(Component, Clone, Debug)]
pub struct Burst {
    /// Shortest and longest burst, in world units.
    pub min_run: f32,
    pub max_run: f32,
    pub speed_boost: f32,
    /// Inside this distance a pinned target gets a step-in spread instead.
    pub close_range: f32,
    run: StraightRun,
    last_side: f32,
}

impl Burst {
    /// Bursts `min_run..max_run` on every completed cast.
    pub fn new(min_run: f32, max_run: f32, speed_boost: f32, close_range: f32) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            close_range,
            run: StraightRun::default(),
            last_side: 1.0,
        }
    }

    fn is_called(&self) -> bool {
        self.run.called
    }

    fn call(&mut self, direction: Vec2, run: f32) {
        self.run.call(direction, run);
    }

    fn finish(&mut self) {
        self.run.finish();
    }

    /// A fresh tangent that cuts across the target's orbit when it circles, or
    /// flips sides when it holds still. A pinned target inside `close_range`
    /// gets the step-in spread instead of another sidestep.
    fn direction_to(&mut self, target: &TargetView, random: &mut BrainRandom) -> Vec2 {
        if target.distance <= self.close_range
            && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
        {
            let spread = random.range(-BURST_STEP_IN_SPREAD, BURST_STEP_IN_SPREAD);
            return target.bearing.rotate(Vec2::from_angle(spread));
        }
        let side = if target.tangential_velocity.abs() > TARGET_MOVING_SPEED {
            -target.tangential_velocity.signum()
        } else {
            -self.last_side
        };
        self.last_side = side;
        let tangent = target.bearing.perp() * side;
        (tangent + target.bearing * BURST_OUTWARD_BIAS).normalize_or_zero()
    }

    fn step(&mut self, distance: f32) -> Option<Vec2> {
        self.run.step(distance)
    }
}

/// Ice mage's danger-ring escape. Opt-in at spawn; absent means never slides.
///
/// A slide fixes its bearing once and runs straight out on a speed boost, so it
/// reads as a committed escape rather than another spacing adjustment. The
/// cooldown opens the window the player can use to close again.
#[derive(Component, Clone, Debug)]
pub struct Slide {
    /// Shortest and longest slide, in world units.
    pub min_run: f32,
    pub max_run: f32,
    pub speed_boost: f32,
    /// Target distance that calls a slide.
    pub danger_range: f32,
    /// Seconds after a slide before another can be called.
    pub cooldown: f32,
    run: StraightRun,
    rearm_at: f32,
}

impl Slide {
    /// Slides `min_run..max_run` whenever a target crosses `danger_range`,
    /// rearming `cooldown` seconds after the last slide ends.
    pub fn new(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        danger_range: f32,
        cooldown: f32,
    ) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            danger_range,
            cooldown,
            run: StraightRun::default(),
            rearm_at: 0.0,
        }
    }

    fn is_called(&self) -> bool {
        self.run.called
    }

    fn ready(&self, now: f32) -> bool {
        now >= self.rearm_at
    }

    fn call(&mut self, direction: Vec2, run: f32) {
        self.run.call(direction, run);
    }

    /// Clears a slide that lost the mode or its target. Only a spent run arms
    /// the cooldown (see `step`): a call that never executed must not lock out
    /// the next escape.
    fn finish(&mut self) {
        self.run.finish();
    }


    fn step(&mut self, distance: f32, now: f32) -> Option<Vec2> {
        if !self.run.called {
            return None;
        }
        let direction = self.run.step(distance);
        if !self.run.called {
            self.rearm_at = now + self.cooldown;
        }
        direction
    }
}

/// Latched mana-break state for casters. A single threshold would flicker on
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

    /// Updates the latch from the current mana fraction and reports the break.
    fn update(&mut self, mana_fraction: f32) -> bool {
        let mana_fraction = mana_fraction.clamp(0.0, 1.0);
        if self.recovering {
            if mana_fraction >= self.resume_at {
                self.recovering = false;
            }
        } else if mana_fraction <= self.trigger_at {
            self.recovering = true;
        }
        self.recovering
    }
}

/// Brief plant while a caster's equipment is in use. Opt-in at spawn; absent
/// means the character keeps moving through its cast.
///
/// The plant is measured from the start of the attack, so a cast shorter than
/// `hold` leaves a readable recovery beat instead of sliding straight into the
/// next move.
#[derive(Component, Clone, Copy, Debug)]
pub struct CastPlant {
    pub hold: f32,
    hold_until: f32,
    was_attacking: bool,
}

impl CastPlant {
    /// Plants for `hold` seconds starting when the caster begins an attack.
    pub fn new(hold: f32) -> Self {
        Self {
            hold,
            hold_until: 0.0,
            was_attacking: false,
        }
    }

    /// Returns whether the caster should stay planted this frame.
    fn update(&mut self, attacking: bool, now: f32) -> bool {
        if attacking && !self.was_attacking {
            self.hold_until = now + self.hold;
        }
        self.was_attacking = attacking;
        attacking || now < self.hold_until
    }
}

/// Per-tick equipment intent, written by `run_decisions` when present.
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
/// Whether the brain wants to drink a carried potion this tick.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WantsToHeal(pub bool);
/// Whether the brain wants to loose its innate breath this tick.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct WantsToBreathe(pub bool);

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

/// Per-entity randomness, seeded from the root seed and the entity.
#[derive(Component, Default)]
pub struct BrainRandom(Option<ChaCha8Rng>);

impl BrainRandom {
    #[cfg(test)]
    pub fn seeded(seed: u64) -> Self {
        Self(Some(ChaCha8Rng::seed_from_u64(seed)))
    }

    fn ensure_seeded(&mut self, entity: Entity, root_seed: u64) {
        if self.0.is_none() {
            let entity_seed = entity.to_bits().wrapping_mul(0x9E37_79B9_7F4A_7C15);
            self.0 = Some(ChaCha8Rng::seed_from_u64(root_seed ^ entity_seed));
        }
    }

    fn generator(&mut self) -> &mut ChaCha8Rng {
        self.0
            .as_mut()
            .expect("brain randomness is seeded before it is read")
    }

    /// Uniform value in `min..max`, or `min` for an empty/invalid range.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        if max > min {
            self.generator().random_range(min..max)
        } else {
            min
        }
    }

    /// A uniformly random unit direction.
    pub fn direction(&mut self) -> Vec2 {
        let angle = self.generator().random_range(0.0..std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }
}

/// Rejects gated options on a brain without a distance band: `pass_gate` would
/// otherwise treat every gate as open and silently ignore the eligibility rule.
fn validate_band_usage(kind: BrainKind) -> Result<(), String> {
    let definition = kind.definition();
    let gated = definition
        .motion_options
        .iter()
        .any(|option| !option.gates.is_empty())
        || definition
            .slot_options
            .iter()
            .any(|options| options.iter().any(|option| !option.gates.is_empty()))
        || definition
            .heal_options
            .iter()
            .any(|option| !option.gates.is_empty())
        || definition
            .breath_options
            .iter()
            .any(|option| !option.gates.is_empty());
    if gated && kind.band().is_none() {
        return Err("gated options require a distance band".to_string());
    }
    Ok(())
}

/// Debug-only guard against a spawned steering band drifting away from the
/// band its brain's gates measure against.
#[cfg(debug_assertions)]
fn validate_spawned_bands(characters: Query<(&BrainKind, &Strafe), Added<BrainKind>>) {
    for (kind, strafe) in &characters {
        if let Some(band) = kind.band() {
            debug_assert_eq!(
                strafe.band, band,
                "{kind:?} spawned with a Strafe band that does not match its gates"
            );
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    for &kind in BrainKind::ALL {
        if let Err(error) = validate_brain(kind.definition()) {
            panic!("{kind:?} brain is invalid: {error}");
        }
        if let Err(error) = validate_band_usage(kind) {
            panic!("{kind:?} brain is invalid: {error}");
        }
    }

    app.init_resource::<DecisionClock>()
        .init_resource::<AiSeed>()
        .add_systems(Startup, seed_ai_seed)
        .add_systems(
            Update,
            run_decisions
                .after(crate::combat::mana::regenerate_mana)
                .after(tick_equippable_cooldown)
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            (apply_slot_requests, apply_heal_requests).in_set(AiSet::Execute),
        )
        .add_systems(
            Update,
            drive_motion.in_set(AiSet::Motion).after(apply_slowed),
        );

    #[cfg(debug_assertions)]
    app.add_systems(Update, validate_spawned_bands);
}

fn seed_ai_seed(map_rng: Option<Res<MapRng>>, mut ai_seed: ResMut<AiSeed>) {
    if let Some(map_rng) = map_rng {
        ai_seed.0 = map_rng.root_seed();
    } else {
        warn!("MapRng is missing; seeding AI randomness with 0");
    }
}

fn run_decisions(
    time: Res<Time<Virtual>>,
    mut clock: ResMut<DecisionClock>,
    ai_seed: Res<AiSeed>,
    equipment: EquipmentRead,
    mut brains: Query<(
        Entity,
        &BrainKind,
        &mut MotionState,
        Has<Targeting>,
        Option<&TargetInfo>,
        &Transform,
        Option<&Health>,
        Option<&Anchor>,
        Option<&mut Swoop>,
        Option<&mut Kite>,
        Option<&mut AttackState>,
        (
            Option<&mut SlotIntent>,
            Option<&mut WantsToHeal>,
            Option<&mut WantsToBreathe>,
        ),
        &mut BrainRandom,
        (
            Option<&Mana>,
            Option<&mut ManaRecovery>,
            Option<&mut Burst>,
            Option<&mut Slide>,
        ),
    )>,
) {
    if !clock.0.tick(time.delta()).just_finished() {
        return;
    }

    let now = time.elapsed_secs();

    for (
        entity,
        kind,
        mut motion,
        has_target,
        target_info,
        transform,
        health,
        anchor,
        mut swoop,
        mut kite,
        mut attack_state,
        (mut slot_intent, mut heal_intent, mut breathe_intent),
        mut random,
        (mana, mut recovery, mut burst, mut slide),
    ) in &mut brains
    {
        random.ensure_seeded(entity, ai_seed.0);

        let target = if has_target {
            target_info.and_then(|info| {
                info.distance.is_finite().then(|| TargetView {
                    distance: info.distance,
                    line_of_sight: info.line_of_sight,
                    bearing: info.direction,
                    velocity: info.target_velocity,
                    tangential_velocity: info.target_velocity.dot(info.direction.perp()),
                })
            })
        } else {
            None
        };

        let mut slot_ready = [false; EquipmentSlot::ALL.len()];
        for slot in EquipmentSlot::ALL {
            slot_ready[slot as usize] = equipment.can_use(entity, slot).is_ok();
        }

        let mana_fraction = mana.and_then(Mana::fraction).unwrap_or(1.0);
        let mana_recovering = recovery
            .as_deref_mut()
            .is_some_and(|recovery| recovery.update(mana_fraction));

        // A warrior that just finished a swing usually breaks away. Roll once
        // per completed swing, then fix a bearing inside the away cone and run
        // it out; the slot request is suppressed until the break ends. Losing
        // the target cancels the break so a stale call cannot re-arm it.
        let attack_finished = attack_state
            .as_ref()
            .is_some_and(|state| state.finished_since_decision);
        let kite_called = if let Some(kite) = kite.as_deref_mut() {
            if kite.is_called() && target.is_none() {
                kite.finish();
            }
            if attack_finished
                && target.is_some()
                && let Some(info) = target_info
                && random.range(0.0, 1.0) < kite.chance
            {
                let away = (-info.direction).normalize_or_zero();
                let bearing = if away.length_squared() > f32::EPSILON {
                    away.rotate(Vec2::from_angle(random.range(-kite.spread, kite.spread)))
                } else {
                    random.direction()
                };
                kite.call(bearing, random.range(kite.min_run, kite.max_run));
            }
            kite.is_called()
        } else {
            false
        };

        // A finished fire volley asks for one tangent hop. The bearing is
        // fixed here so the mage commits to a new firing angle; a latched call
        // is dropped when the target disappears.
        let burst_called = if let Some(burst) = burst.as_deref_mut() {
            if burst.is_called() && target.is_none() {
                burst.finish();
            }
            if attack_finished
                && !mana_recovering
                && let Some(target) = target
            {
                let direction = burst.direction_to(&target, &mut random);
                if direction.length_squared() > f32::EPSILON {
                    burst.call(direction, random.range(burst.min_run, burst.max_run));
                }
            }
            burst.is_called()
        } else {
            false
        };
        // The latch is one decision tick wide: every consumer above has seen
        // it, so clear it until the next attack actually finishes.
        if let Some(state) = attack_state.as_deref_mut() {
            state.finished_since_decision = false;
        }

        // Crossing the ice mage's danger ring calls one straight slide. The
        // cooldown is armed when the run is spent, so holding the ring does not
        // refresh the escape every decision tick.
        let slide_called = if let Some(slide) = slide.as_deref_mut() {
            if slide.is_called() && target.is_none() {
                slide.finish();
            }
            if !slide.is_called()
                && let Some(target) = target
                && target.distance <= slide.danger_range
                && slide.ready(now)
            {
                let direction = if target.bearing.length_squared() > f32::EPSILON {
                    -target.bearing
                } else {
                    random.direction()
                };
                slide.call(direction, random.range(slide.min_run, slide.max_run));
            }
            slide.is_called()
        } else {
            false
        };

        // The calm cycle always opens at idle. Once the current half's
        // commitment window expires, ask for the other half.
        let mut calm_phase = motion.calm_phase.unwrap_or(MotionMode::Idle);
        if motion.mode.is_calm() && motion.committed_until > 0.0 && now >= motion.committed_until {
            calm_phase = motion.mode.next_calm();
        }

        // Swoop lottery: while circling or keeping distance, roll at most once
        // per `interval`. A miss schedules the next roll immediately; a hit
        // latches the call and leaves the timer untouched until `finish` starts
        // the next interval. That keeps the full pause after a dive instead of
        // letting a long pass return to an already-expired timer.
        if let Some(swoop) = swoop.as_deref_mut()
            && matches!(
                motion.mode,
                MotionMode::Strafe | MotionMode::KeepDistance(_)
            )
            && now >= swoop.next_roll_at
        {
            if random.range(0.0, 1.0) < swoop.chance {
                swoop.called = true;
                swoop.started_at = now;
            } else {
                swoop.next_roll_at = now + swoop.interval;
            }
        }
        let context = DecisionContext {
            target,
            slot_ready,
            health_fraction: health.and_then(Health::fraction).unwrap_or(1.0),
            mana_fraction,
            mana_recovering,
            anchor_overshoot_fraction: anchor
                .and_then(|anchor| anchor.overshoot_fraction(transform.translation.xy())),
            anchor_arrived: anchor.map(|anchor| {
                anchor.distance_from(transform.translation.xy()) <= ANCHOR_ARRIVAL_RADIUS
            }),
            calm_phase,
            strafe_band: kind.band(),
            swoop_called: swoop.as_deref().is_some_and(|swoop| swoop.called),
            kite_called,
            burst_called,
            slide_called,
        };

        let definition = kind.definition();
        // A spent break hands the incumbent slot back to chase: leaving Kite
        // as the incumbent would strand the warrior in the melee deadband,
        // where neither melee option is allowed to enter.
        let current_mode = if motion.mode == MotionMode::Kite && !kite_called {
            MotionMode::Chase
        } else {
            motion.mode
        };
        // Shape 3: the utility proposal is memoryless; the arbiter answers
        // "is now a good time to actually switch?" Same-group jitter waits
        // out the dwell or loses on the challenger margin, while group
        // changes and an ineligible incumbent go through at once.
        let (decision, accepted) =
            decide_with_arbiter(definition, &context, current_mode, motion.dwell_until - now);
        let changed = accepted != motion.mode;
        if changed {
            motion.mode = accepted;
            motion.dwell_until = now + MIN_DWELL_SECS;
        }
        // Arm the calm-cycle window on entry, and once on spawn when the
        // initial mode is already calm. An expired window is not re-armed
        // while the mode holds: expiry is what asks the phase to advance.
        let unarmed_calm = accepted.is_calm() && motion.committed_until <= 0.0;
        if changed || unarmed_calm {
            motion.committed_until = commit_for(definition, accepted)
                .map_or(now, |window| now + random.range(window.min(), window.max()));
        }

        // An ended dive or a call that never converted clears here, so the
        // next lottery starts from an idle swoop instead of a stale latch.
        // `finish` also restarts the roll timer and, for a real dive, the exit
        // recovery window.
        if accepted != MotionMode::Swoop
            && let Some(swoop) = swoop.as_deref_mut()
        {
            swoop.finish(now);
        }
        // A break that lost the mode to something else (a flee, say) is
        // dropped here so a stale call cannot fire again later.
        if accepted != MotionMode::Kite
            && let Some(kite) = kite.as_deref_mut()
        {
            kite.finish();
        }
        // The same rule keeps a fire burst or ice slide from surviving a mode
        // change it lost. Only a spent run arms the slide cooldown (see
        // `Slide::step`), so clearing a call that never executed stays free.
        if accepted != MotionMode::Burst
            && let Some(burst) = burst.as_deref_mut()
        {
            burst.finish();
        }
        if accepted != MotionMode::Slide
            && let Some(slide) = slide.as_deref_mut()
        {
            slide.finish();
        }
        // A reactive mode hands the calm cycle back to idle, which is where the
        // cycle always starts.
        let calm_phase = Some(if accepted.is_calm() {
            accepted
        } else {
            MotionMode::Idle
        });
        if motion.calm_phase != calm_phase {
            motion.calm_phase = calm_phase;
        }

        if let Some(slot_intent) = slot_intent.as_deref_mut() {
            let intents = decision.slot_intents;
            if slot_intent.0 != intents {
                slot_intent.set_all(intents);
            }
        }
        if let Some(heal_intent) = heal_intent.as_deref_mut() {
            if heal_intent.0 != decision.wants_heal {
                heal_intent.0 = decision.wants_heal;
            }
        }
        if let Some(breathe_intent) = breathe_intent.as_deref_mut() {
            if breathe_intent.0 != decision.wants_breathe {
                breathe_intent.0 = decision.wants_breathe;
            }
        }
    }
}

/// Re-gates stale intent on the live target state; skips the dead. Decisions
/// price range at 10 Hz but this runs every frame, so a target that drifted
/// out of reach between ticks must not keep a stale `try_use` alive.
fn apply_slot_requests(
    mut equipment: EquipmentUsage,
    requests: Query<(
        Entity,
        &BrainKind,
        &SlotIntent,
        Has<Targeting>,
        Option<&TargetInfo>,
        Option<&Health>,
    )>,
) {
    for (entity, kind, slot_intent, has_target, target_info, health) in &requests {
        if health.is_some_and(|health| health.hp <= 0.0) {
            continue;
        }
        // Live range check mirrors the slot considerations: warriors swing at
        // melee reach (plus the hold gate's hysteresis), casters fire inside
        // the cast band. Anything else keeps its decision-tick intent.
        let in_range = match kind {
            BrainKind::Warrior => target_info.is_some_and(|info| {
                info.distance.is_finite()
                    && info.distance <= MELEE_RANGE + MELEE_HYSTERESIS
            }),
            BrainKind::FireMage | BrainKind::IceMage => target_info.is_some_and(|info| {
                info.distance.is_finite() && info.distance <= CAST_MAX_RANGE
            }),
            BrainKind::Villager | BrainKind::Bat => true,
        };
        for slot in EquipmentSlot::ALL {
            let sighted = target_info.is_some_and(|info| info.line_of_sight);
            let requested = has_target && sighted && in_range && slot_intent.get(slot);
            let in_use = equipment.is_using(entity, slot);

            if requested && !in_use {
                if equipment.can_use(entity, slot).is_ok() {
                    let _ = equipment.try_use(entity, slot);
                }
            } else if !requested && in_use && !slot.is_instant_use() {
                equipment.stop_using(entity, slot);
            }
        }
    }
}
/// Drinks a carried potion straight from the inventory; skips the dead.
fn apply_heal_requests(
    mut commands: Commands,
    requests: Query<(Entity, &WantsToHeal, Option<&Items>, Option<&Health>)>,
    consumables: Query<(), With<Consumable>>,
) {
    for (entity, heal_intent, items, health) in &requests {
        if !heal_intent.0 || health.is_some_and(|health| health.hp <= 0.0) {
            continue;
        }
        let potion = items
            .iter()
            .flat_map(|items| items.iter())
            .find(|item| consumables.contains(*item));
        if let Some(potion) = potion {
            commands.trigger(Consume {
                entity,
                item_entity: potion,
            });
        }
    }
}

fn pick_wander_direction(
    random: &mut BrainRandom,
    anchor: Option<&Anchor>,
    position: Vec2,
) -> Vec2 {
    let candidate = random.direction();
    let Some(anchor) = anchor else {
        return candidate;
    };
    if anchor
        .overshoot_fraction(position)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let home = anchor.origin() - position;
    if home == Vec2::ZERO {
        return candidate;
    }
    home.normalize_or_zero()
        .rotate(Vec2::from_angle(random.range(-0.6, 0.6)))
}

/// Post-dive speed boost and whether the recovery window is still open.
fn swoop_recovery(swoop: Option<&Swoop>, now: f32) -> (f32, bool) {
    let boost = swoop.map_or(1.0, |swoop| swoop.recovery_boost(now));
    (boost, boost > 1.0)
}

fn drive_motion(
    time: Res<Time<Virtual>>,
    ai_seed: Res<AiSeed>,
    mut characters: Query<
        (
            Entity,
            &MotionState,
            &mut SimpleMotion,
            &Transform,
            Option<&TargetInfo>,
            Has<Targeting>,
            Option<&Anchor>,
            Option<&mut Wander>,
            Option<&mut Strafe>,
            Option<&mut Swoop>,
            Option<&mut Kite>,
            &mut BrainRandom,
            Option<&TurnRate>,
            (
                Option<&AttackState>,
                Option<&mut CastPlant>,
                Option<&mut Burst>,
                Option<&mut Slide>,
            ),
        ),
        With<BrainKind>,
    >,
) {
    let now = time.elapsed_secs();

    for (
        entity,
        motion_state,
        mut motion,
        transform,
        target_info,
        has_target,
        anchor,
        mut wander,
        mut strafe,
        mut swoop,
        mut kite,
        mut random,
        turn_rate,
        (attack_state, mut cast_plant, mut burst, mut slide),
    ) in &mut characters
    {
        random.ensure_seeded(entity, ai_seed.0);
        let turn_cap = turn_rate.map(|rate| rate.0 * time.delta_secs());
        // Shared steering input, read before any arm borrows motion: the arms
        // that steer toward a world point read it here.
        let position = transform.translation.xy();
        // A dive's speed boost fades after the pass, and the orbit holds still
        // during that window so the entry back into the circle does not also
        // flip the tangent.
        let (recovery_boost, recovering) = swoop_recovery(swoop.as_deref(), now);

        // Casters plant through the read of their own wind-up. Keeping this
        // ahead of the steering match also means a committed burst or slide
        // cannot drag the caster out from under a staff that is already firing.
        let attacking = attack_state.is_some_and(|state| state.is_attacking);
        if cast_plant
            .as_deref_mut()
            .is_some_and(|plant| plant.update(attacking, now))
        {
            if !motion.is_idle() {
                motion.stop_moving();
            }
            continue;
        }

        if motion_state.mode.needs_target() && (!has_target || target_info.is_none()) {
            if !motion.is_idle() {
                motion.stop_moving();
            }
            continue;
        }

        match motion_state.mode {
            MotionMode::Idle | MotionMode::Hold => {
                if !motion.is_idle() {
                    motion.stop_moving();
                }
            }
            MotionMode::Chase => {
                drive_direction(&mut motion, chase_direction(target_info), 1.0, turn_cap);
            }
            // Still in the fight: run from the target, not toward home.
            // Going home happens only once the target is lost.
            MotionMode::Flee => {
                drive_direction(&mut motion, flee_direction(target_info), 1.0, turn_cap);
            }
            // A break runs a fixed bearing, so the warrior keeps its chosen
            // line even as the target follows; normal chase turns it back once
            // the run is spent.
            MotionMode::Kite => {
                let run_step = if motion.is_stunned() {
                    0.0
                } else {
                    motion.max_speed * motion.speed_scale() * time.delta_secs()
                };
                let direction = kite.as_deref_mut().and_then(|kite| kite.step(run_step));
                drive_direction(&mut motion, direction, 1.0, turn_cap);
            }
            // A burst and a slide share the same straight-run contract, but
            // keep separate call timers: one is an after-volley hop, the other
            // a danger-ring escape.
            MotionMode::Burst => {
                let speed_boost = burst.as_deref().map_or(1.0, |burst| burst.speed_boost);
                let run_step = if motion.is_stunned() {
                    0.0
                } else {
                    motion.max_speed * speed_boost * motion.speed_scale() * time.delta_secs()
                };
                let direction = burst.as_deref_mut().and_then(|burst| burst.step(run_step));
                drive_boosted(&mut motion, direction, speed_boost, turn_cap);
            }
            MotionMode::Slide => {
                let speed_boost = slide.as_deref().map_or(1.0, |slide| slide.speed_boost);
                let run_step = if motion.is_stunned() {
                    0.0
                } else {
                    motion.max_speed * speed_boost * motion.speed_scale() * time.delta_secs()
                };
                let direction = slide
                    .as_deref_mut()
                    .and_then(|slide| slide.step(run_step, now));
                drive_boosted(&mut motion, direction, speed_boost, turn_cap);
            }
            MotionMode::ReturnToAnchor => drive_direction(
                &mut motion,
                anchor_direction(position, anchor),
                1.0,
                turn_cap,
            ),
            MotionMode::KeepDistance(band) => drive_boosted(
                &mut motion,
                keep_distance_direction(target_info, band, wander.as_deref_mut(), now, &mut random),
                recovery_boost,
                turn_cap,
            ),
            MotionMode::Strafe => drive_boosted(
                &mut motion,
                strafe_direction(
                    target_info,
                    strafe.as_deref_mut(),
                    &mut random,
                    now,
                    recovering,
                ),
                recovery_boost,
                turn_cap,
            ),
            MotionMode::Swoop => {
                let heading = motion.direction;
                let speed_boost = if recovering {
                    recovery_boost
                } else {
                    swoop
                        .as_deref()
                        .map_or(1.0, |swoop| 1.0 + swoop.speed_boost)
                };
                // Stunned biters hold their place on the arc; the safety bound
                // finishes the dive once they recover or never do.
                let flight_distance = if motion.is_stunned() {
                    0.0
                } else {
                    motion.max_speed * speed_boost * motion.speed_scale() * time.delta_secs()
                };
                let direction = swoop_direction(
                    target_info,
                    position,
                    heading,
                    swoop.as_deref_mut(),
                    flight_distance,
                    now,
                );
                // The bite applier can finish the pass in Execute, before the
                // next decision changes the mode. Start folding into the circle
                // on that same frame instead of stopping until the decision.
                let direction = if direction.is_none() && recovering {
                    strafe_direction(target_info, strafe.as_deref_mut(), &mut random, now, true)
                } else {
                    direction
                };
                drive_boosted(&mut motion, direction, speed_boost, turn_cap);
            }
            MotionMode::Wander => {
                let speed_scale = wander
                    .as_deref()
                    .map_or(WANDER_SPEED_SCALE, |w| w.speed_scale);
                drive_direction(
                    &mut motion,
                    wander_direction(wander.as_deref_mut(), anchor, position, now, &mut random),
                    speed_scale,
                    turn_cap,
                );
            }
        }
    }
}

fn drive_direction(
    motion: &mut SimpleMotion,
    direction: Option<Vec2>,
    speed_scale: f32,
    turn_cap: Option<f32>,
) {
    match direction {
        Some(desired) => {
            let heading = turn_towards(motion, desired, turn_cap);
            motion.start_moving_scaled(heading, speed_scale);
        }
        None => {
            if !motion.is_idle() {
                motion.stop_moving();
            }
        }
    }
}
fn drive_boosted(
    motion: &mut SimpleMotion,
    direction: Option<Vec2>,
    speed_boost: f32,
    turn_cap: Option<f32>,
) {
    match direction {
        Some(desired) => {
            let heading = turn_towards(motion, desired, turn_cap);
            motion.start_moving_boosted(heading, speed_boost);
        }
        None => {
            if !motion.is_idle() {
                motion.stop_moving();
            }
        }
    }
}
fn turn_towards(motion: &SimpleMotion, desired: Vec2, turn_cap: Option<f32>) -> Vec2 {
    if let Some(max_angle) = turn_cap
        && motion.direction.length_squared() > f32::EPSILON
        && desired.length_squared() > f32::EPSILON
    {
        motion.direction.rotate_towards(desired, max_angle)
    } else {
        desired
    }
}
fn chase_direction(target_info: Option<&TargetInfo>) -> Option<Vec2> {
    let direction = target_info.map_or(Vec2::ZERO, |info| info.direction);
    (direction.length_squared() > f32::EPSILON).then_some(direction)
}
fn flee_direction(target_info: Option<&TargetInfo>) -> Option<Vec2> {
    chase_direction(target_info).map(|direction| -direction)
}
fn anchor_direction(position: Vec2, anchor: Option<&Anchor>) -> Option<Vec2> {
    let anchor = anchor?;
    let to_anchor = anchor.origin() - position;
    (to_anchor.length() > ANCHOR_ARRIVAL_RADIUS).then_some(to_anchor.normalize_or_zero())
}
fn strafe_direction(
    target_info: Option<&TargetInfo>,
    strafe: Option<&mut Strafe>,
    random: &mut BrainRandom,
    now: f32,
    hold_orbit: bool,
) -> Option<Vec2> {
    let info = target_info?;
    let direction = chase_direction(Some(info))?;
    let strafe = strafe?;
    // Do not reroll the orbit side or standoff while the biter is settling out
    // of a dive; a flip here would reverse the tangent at the handoff.
    let orbit_due = now >= strafe.reroll_at;
    if strafe.orbit.is_none() || (!hold_orbit && orbit_due) {
        let orbit = if strafe.smart_orbit {
            smart_strafe_orbit(info, direction, strafe.orbit, random)
        } else {
            match strafe.orbit {
                Some(orbit) if random.range(0.0, 1.0) < strafe.switch_chance => orbit.flipped(),
                Some(orbit) => orbit,
                None if random.range(0.0, 1.0) < 0.5 => Orbit::Clockwise,
                None => Orbit::CounterClockwise,
            }
        };
        strafe.orbit = Some(orbit);
        if strafe.standoff_range > 0.0 {
            strafe.standoff = random.range(-strafe.standoff_range, strafe.standoff_range);
        }
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    } else if hold_orbit && orbit_due {
        // A dive can outlive the orbit's reroll clock. Start a fresh window at
        // the handoff instead of flipping immediately after the recovery.
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    }
    let tangent = direction.perp() * strafe.orbit.unwrap_or_default().sign();
    // Positive error means too far from the shifted ring, so the correction
    // points at the target.
    let radial =
        direction * strafe.band.center_error(info.distance - strafe.standoff) * strafe.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}

/// Picks the orbit side that moves against the target's tangential motion.
/// Radial or still targets keep the current side so the mage does not jitter.
fn smart_strafe_orbit(
    info: &TargetInfo,
    direction: Vec2,
    current: Option<Orbit>,
    random: &mut BrainRandom,
) -> Orbit {
    let tangential_motion = info.target_velocity.dot(direction.perp());
    if tangential_motion.abs() > SMART_STRAFE_INPUT_DEADBAND {
        if tangential_motion > 0.0 {
            Orbit::Clockwise
        } else {
            Orbit::CounterClockwise
        }
    } else if let Some(orbit) = current {
        orbit
    } else if random.range(0.0, 1.0) < 0.5 {
        Orbit::Clockwise
    } else {
        Orbit::CounterClockwise
    }
}
fn swoop_direction(
    target_info: Option<&TargetInfo>,
    position: Vec2,
    heading: Vec2,
    swoop: Option<&mut Swoop>,
    flight_distance: f32,
    now: f32,
) -> Option<Vec2> {
    let info = target_info?;
    let swoop = swoop?;
    // Only a non-finite distance blocks the pass: a collapsed (zero-bearing)
    // snapshot builds the zero-length arc `SwoopPass::new` already handles,
    // which clears on the next bite tick instead of stranding the latch with
    // no timeout owner.
    if !info.distance.is_finite() || !swoop.called {
        return None;
    }
    let live_target = position + info.direction * info.distance;
    if swoop.pass.is_none() {
        swoop.pass = Some(SwoopPass::new(
            position,
            live_target,
            info.distance,
            heading,
            swoop.exit_turn,
        ));
        swoop.started_at = now;
        swoop.bitten = false;
    }
    let pass = swoop.pass.as_mut()?;
    pass.advance(flight_distance);
    pass.track(live_target, flight_distance * SWOOP_TRACK_RATIO);
    let to_lead = pass.lead(SWOOP_LOOKAHEAD) - position;
    (to_lead.length_squared() > f32::EPSILON).then(|| to_lead.normalize_or_zero())
}
/// Holds the band; a small deadband keeps the arrival from flickering.
fn keep_distance_direction(
    target_info: Option<&TargetInfo>,
    band: DistanceBand,
    wander: Option<&mut Wander>,
    now: f32,
    random: &mut BrainRandom,
) -> Option<Vec2> {
    let distance = target_info.map_or(0.0, |info| info.distance);
    let direction = target_info.map_or(Vec2::ZERO, |info| info.direction);
    if direction.length_squared() <= f32::EPSILON {
        if distance < band.min() {
            let wander = wander?;
            if now >= wander.next_direction_at {
                wander.direction = random.direction();
                wander.next_direction_at = now + 0.2;
            }
            return Some(wander.direction);
        }
        return None;
    }
    let error = band.error(distance);
    if error.abs() < KEEP_ARRIVAL_DEADBAND {
        None
    } else if error < 0.0 {
        Some(-direction)
    } else {
        Some(direction)
    }
}
fn wander_direction(
    wander: Option<&mut Wander>,
    anchor: Option<&Anchor>,
    position: Vec2,
    now: f32,
    random: &mut BrainRandom,
) -> Option<Vec2> {
    let wander = wander?;
    if now >= wander.next_direction_at {
        wander.direction = pick_wander_direction(random, anchor, position);
        wander.next_direction_at = now + random.range(wander.min, wander.max);
    }
    Some(wander.direction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn every_brain_kind_is_valid() {
        for &kind in BrainKind::ALL {
            validate_brain(kind.definition())
                .unwrap_or_else(|error| panic!("{kind:?} brain is invalid: {error}"));
            validate_band_usage(kind)
                .unwrap_or_else(|error| panic!("{kind:?} brain is invalid: {error}"));
        }
    }

    #[test]
    fn fixed_seed_repeats_identically() {
        let mut first = BrainRandom::seeded(7);
        let mut second = BrainRandom::seeded(7);

        for _ in 0..32 {
            assert_eq!(first.direction(), second.direction());
            assert_eq!(
                first.range(0.5, 1.5).to_bits(),
                second.range(0.5, 1.5).to_bits()
            );
        }
    }
    #[test]
    fn wander_direction_bends_home_near_the_edge() {
        let anchor = Anchor::new(Vec2::ZERO, 100.0);
        let mut random = BrainRandom::seeded(3);
        let inner = pick_wander_direction(&mut random, Some(&anchor), Vec2::new(10.0, 10.0));
        assert!((inner.length() - 1.0).abs() < 1e-5);

        // Past the edge every roll must point home-ish, never away.
        let position = Vec2::new(150.0, 20.0);
        let home = (Vec2::ZERO - position).normalize();
        for _ in 0..16 {
            let direction = pick_wander_direction(&mut random, Some(&anchor), position);
            assert!(direction.dot(home) > 0.0);
        }

        let free = pick_wander_direction(&mut random, None, Vec2::new(500.0, 0.0));
        assert!((free.length() - 1.0).abs() < 1e-5);
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
        let bat = world
            .spawn((
                BrainKind::Bat,
                MotionState {
                    mode: MotionMode::Strafe,
                    ..Default::default()
                },
                Targeting(target),
                TargetInfo {
                    distance: CAST_BAND.max() - 50.0,
                    direction: Vec2::X,
                    target_velocity: Vec2::ZERO,
                    line_of_sight: true,
                    in_vision_cone: true,
                },
                Transform::default(),
                Strafe::new(CAST_BAND),
                Swoop::new(chance, interval, 6.0),
                BrainRandom::default(),
            ))
            .id();
        (world, bat)
    }
    fn bat_mode(world: &mut World, bat: Entity) -> MotionMode {
        world.get::<MotionState>(bat).expect("brained bat").mode
    }
    /// The full loop holds: a called tick proposes the dive and the arbiter
    /// accepts it out of the circle the same tick.
    #[test]
    fn swoop_lottery_drives_a_dive() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        world.run_system_once(run_decisions).expect("decisions run");
        assert_eq!(bat_mode(&mut world, bat), MotionMode::Swoop);
    }

    /// The accepted dive persists without re-rolling: the latched call keeps
    /// the option scoring until something finishes it. A one-tick local here
    /// would drop back to Strafe on the very next tick.
    #[test]
    fn swoop_dive_outlives_the_calling_tick() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        for _ in 0..5 {
            world
                .resource_mut::<Time<Virtual>>()
                .advance_by(std::time::Duration::from_millis(110));
            world.run_system_once(run_decisions).expect("decisions run");
            assert_eq!(bat_mode(&mut world, bat), MotionMode::Swoop);
        }
    }

    /// The roll interval blocks the next attempt: after the dive ends
    /// naturally, the latch clears with it and the scorer holds the strafe
    /// until the timer comes due again, even with a guaranteed chance.
    #[test]
    fn swoop_interval_blocks_recall() {
        let (mut world, bat) = swooping_bat(1.0, 100.0);
        world.run_system_once(run_decisions).expect("decisions run");
        assert_eq!(bat_mode(&mut world, bat), MotionMode::Swoop);
        // The bite lands: `finish` clears the call latch, so the next tick
        // prices the option at zero and exits to the circle.
        let now = world.resource::<Time<Virtual>>().elapsed_secs();
        world.get_mut::<Swoop>(bat).expect("diving bat").finish(now);
        world
            .resource_mut::<Time<Virtual>>()
            .advance_by(std::time::Duration::from_millis(110));
        world.run_system_once(run_decisions).expect("decisions run");
        assert_eq!(bat_mode(&mut world, bat), MotionMode::Strafe);
        // Two more full ticks: the next roll is 100 s away, so the timer
        // stays closed even with a guaranteed chance.
        for _ in 0..2 {
            world
                .resource_mut::<Time<Virtual>>()
                .advance_by(std::time::Duration::from_millis(110));
            world.run_system_once(run_decisions).expect("decisions run");
            assert_eq!(bat_mode(&mut world, bat), MotionMode::Strafe);
        }
    }
    /// Counts accepted dives over ticks of circling. Returns the bat to the
    /// circle whenever it leaves the swoop so the lottery can fire again.
    fn dive_count(world: &mut World, bat: Entity, ticks: usize, step_ms: u64) -> usize {
        let mut dives = 0;
        let mut was_swoop = false;
        for _ in 0..ticks {
            world
                .resource_mut::<Time<Virtual>>()
                .advance_by(std::time::Duration::from_millis(step_ms));
            (&mut *world)
                .run_system_once(run_decisions)
                .expect("decisions run");
            let mode = bat_mode(&mut *world, bat);
            if mode == MotionMode::Swoop && !was_swoop {
                dives += 1;
            }
            was_swoop = mode == MotionMode::Swoop;
            if mode == MotionMode::Swoop {
                // The bite applier would finish the dive in game; here the
                // cleared latch alone must make the next tick leave.
                let now = world.resource::<Time<Virtual>>().elapsed_secs();
                world.get_mut::<Swoop>(bat).expect("diving bat").finish(now);
            }
        }
        dives
    }
    /// A stably circling bat dives repeatedly: 35% per roll, one roll every
    /// 2 s, plus the dive time, gives many chances across two minutes.
    #[test]
    fn swoop_fires_repeatedly_while_circling() {
        let (mut world, bat) = swooping_bat(0.35, 2.0);
        assert!(dive_count(&mut world, bat, 1200, 100) >= 3);
    }
    /// Violent chase/strafe churn across the hold band must not starve a
    /// committed strike: flips every 0.2 s keep the dwell permanently fresh.
    /// The first 100 ticks are warmup past the spawn transient; only dives
    /// after that count, so a tick-zero accept cannot fake the result.
    #[test]
    fn swoop_breaks_through_boundary_churn() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        let inside = CAST_BAND.max() - 15.0;
        let outside = CAST_BAND.max() + 10.0;
        for i in 0..100 {
            set_distance(
                &mut world,
                bat,
                if (i / 2) % 2 == 0 { inside } else { outside },
            );
            tick_once(&mut world);
        }
        let mut dives = 0;
        let mut was_swoop = bat_mode(&mut world, bat) == MotionMode::Swoop;
        for i in 100..600 {
            set_distance(
                &mut world,
                bat,
                if (i / 2) % 2 == 0 { inside } else { outside },
            );
            tick_once(&mut world);
            let mode = bat_mode(&mut world, bat);
            if mode == MotionMode::Swoop && !was_swoop {
                dives += 1;
            }
            was_swoop = mode == MotionMode::Swoop;
        }
        assert!(dives >= 1, "dwell churn starved every call");
    }
    fn set_distance(world: &mut World, bat: Entity, distance: f32) {
        world
            .get_mut::<TargetInfo>(bat)
            .expect("tracked bat")
            .distance = distance;
    }
    fn tick_once(world: &mut World) {
        world
            .resource_mut::<Time<Virtual>>()
            .advance_by(std::time::Duration::from_millis(100));
        (&mut *world)
            .run_system_once(run_decisions)
            .expect("decisions run");
    }

    /// The committed pass bows through the target snapshot and back out to
    /// the entry radius on the far side, so the dive flies past the target
    /// instead of stopping on it.
    #[test]
    fn swoop_pass_bows_through_the_target_and_back_out() {
        let mut swoop = Swoop::new(1.0, 0.0, 6.0);
        swoop.called = true;
        let target = TargetInfo {
            distance: 250.0,
            direction: Vec2::X,
            target_velocity: Vec2::ZERO,
            line_of_sight: true,
            in_vision_cone: true,
        };
        // The first steered frame builds the arc and aims it at the target.
        let first = swoop_direction(
            Some(&target),
            Vec2::ZERO,
            Vec2::Y,
            Some(&mut swoop),
            0.0,
            0.0,
        )
        .expect("diving");
        assert!(first.dot(Vec2::X) > 0.8);

        let pass = swoop.pass.expect("pass built");
        // The arc's midpoint sits on the target snapshot, and its exit sits on
        // the far side at the entry radius.
        assert!(pass.point(0.5).distance(Vec2::new(250.0, 0.0)) < 1e-3);
        assert!((pass.end.distance(pass.target) - 250.0).abs() < 1e-3);
        assert!(pass.end.x > 250.0);

        // Just past the target is not "back out": the dive keeps flying until
        // the live distance opens back up to the circling radius.
        let just_past = pass.target - pass.entry_bearing * 30.0;
        assert!(!swoop.cleared(just_past, 30.0));
        assert!(swoop.cleared(just_past, 250.0));

        // Flying the whole arc also ends the dive, even if the target chases.
        let _ = swoop_direction(
            Some(&target),
            pass.end,
            Vec2::X,
            Some(&mut swoop),
            pass.length,
            1.0,
        );
        assert!(swoop.cleared(pass.end, 0.0));
    }
}
