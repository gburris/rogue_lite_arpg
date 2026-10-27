//! Per-frame steering: turns the brain's chosen motion into [`SimpleMotion`].

mod anchor;
mod dash;
mod orbit;
mod swoop;
mod wander;

use bevy::{ecs::query::QueryData, prelude::*};

pub use anchor::Anchor;
pub use dash::Dash;
pub use orbit::Orbit;
pub use swoop::Swoop;
pub use wander::Wander;

use super::capability::{Capabilities, Capability, record_capability};
use super::engine::Interval;
use super::perception::Targeting;
use super::{ActiveMotion, Brain, BrainRng};
use crate::character::{state::SimpleMotion, vision::TargetInfo};

const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// How a character moves: a continuous motion that runs until replaced, or a
/// finite committed move that takes exclusive control of steering until its
/// executor reports it finished and is never re-scored while it runs. New behaviors
/// should reuse these motions, not add new variants, unless the
/// steering contract itself is new.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Motion {
    /// Disengaged: alternates standing and ambling, paced by the `Wander`
    /// component.
    #[default]
    Calm,
    Chase,
    ReturnToAnchor,
    /// Wounded break: still in the fight, just getting away from the target.
    Flee,
    KeepDistance(Interval),
    /// Stands still on purpose. Scored like any motion, so it can win the
    /// slot and keep momentum; an empty slot also stands still, but nothing
    /// chose it.
    Idle,
    /// Circles the target inside the distance range while equipment stays hot.
    Orbit(Interval),
    /// Committed straight run; its `Dash` constructor (kite, hop, slide) sets its aim.
    Dash,
    /// Committed dive through a snapshot of the target and back out.
    Swoop,
}

impl Motion {
    /// Every motion in array order. A payload here only stands in; arrays
    /// index by variant.
    pub const ALL: &'static [Self] = &[
        Self::Calm,
        Self::Chase,
        Self::ReturnToAnchor,
        Self::Flee,
        Self::KeepDistance(Interval::new(0.0, 1.0)),
        Self::Idle,
        Self::Orbit(Interval::new(0.0, 1.0)),
        Self::Dash,
        Self::Swoop,
    ];

    /// Number of distinct motions, for per-motion arrays.
    pub const COUNT: usize = Self::ALL.len();

    /// Position of this motion in per-motion arrays, ignoring any distance range.
    pub const fn index(self) -> usize {
        match self {
            Self::Calm => 0,
            Self::Chase => 1,
            Self::ReturnToAnchor => 2,
            Self::Flee => 3,
            Self::KeepDistance(_) => 4,
            Self::Idle => 5,
            Self::Orbit(_) => 6,
            Self::Dash => 7,
            Self::Swoop => 8,
        }
    }

    /// Whether this motion ends on its own and reports it, rather than
    /// running until replaced.
    pub const fn is_finite(self) -> bool {
        match self {
            Self::Dash | Self::Swoop => true,
            Self::Calm
            | Self::Chase
            | Self::ReturnToAnchor
            | Self::Flee
            | Self::KeepDistance(_)
            | Self::Idle
            | Self::Orbit(_) => false,
        }
    }

    /// Whether steering this motion needs a live target.
    pub fn needs_target(self) -> bool {
        match self {
            // Flee is still fighting: the target is live, the goal is just distance.
            Self::Chase
            | Self::KeepDistance(_)
            | Self::Orbit(_)
            | Self::Flee
            | Self::Dash
            | Self::Swoop => true,
            Self::Calm | Self::ReturnToAnchor | Self::Idle => false,
        }
    }

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Calm => "Calm",
            Self::Chase => "Chase",
            Self::ReturnToAnchor => "ReturnToAnchor",
            Self::Flee => "Flee",
            Self::KeepDistance(_) => "KeepDistance",
            Self::Idle => "Idle",
            Self::Orbit(_) => "Orbit",
            Self::Dash => "Dash",
            Self::Swoop => "Swoop",
        }
    }
}

// `ALL` lists every motion in `index` order.
const _: () = {
    // A new variant fails to compile here: add it to this match and to `ALL`.
    match Motion::Calm {
        Motion::Calm
        | Motion::Chase
        | Motion::ReturnToAnchor
        | Motion::Flee
        | Motion::KeepDistance(_)
        | Motion::Idle
        | Motion::Orbit(_)
        | Motion::Dash
        | Motion::Swoop => {}
    }
    let mut position = 0;
    while position < Motion::COUNT {
        assert!(
            Motion::ALL[position].index() == position,
            "Motion::ALL is out of index order"
        );
        position += 1;
    }
};

/// Caps how fast a character turns, in radians per second. Absent means instant turns.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// Extra speed left over from a finite motion's burst, easing off over
/// `duration`. Steering owns it: it multiplies whatever effort the current
/// continuous motion asks for and removes it once it has fully decayed.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct SpeedBoost {
    extra_speed: f32,
    duration: f32,
    ends_at: f32,
    easing: EaseFunction,
}

impl SpeedBoost {
    pub(super) fn new(extra_speed: f32, duration: f32, easing: EaseFunction, now: f32) -> Self {
        Self {
            extra_speed,
            duration,
            ends_at: now + duration,
            easing,
        }
    }

    fn compute_speed_multiplier(&self, now: f32) -> Option<f32> {
        if now >= self.ends_at {
            return None;
        }
        let remaining = ((self.ends_at - now) / self.duration).clamp(0.0, 1.0);
        Some(1.0 + self.extra_speed * self.easing.sample_clamped(remaining))
    }
}

/// A finite motion completed; the brain starts its cooldown and, while `epoch`
/// is still the motion slot's running behavior, ends it and decides again.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
    pub epoch: u32,
}

pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub heading: Vec2,
    pub now: f32,
    pub target_info: Option<&'a TargetInfo>,
    pub anchor: Option<&'a Anchor>,
}

impl SteerInput<'_> {
    pub(super) fn direction_to_target(&self) -> Option<Vec2> {
        let direction = self.target_info.map_or(Vec2::ZERO, |info| info.direction);
        (direction.length_squared() > f32::EPSILON).then_some(direction)
    }

    fn direction_from_target(&self) -> Option<Vec2> {
        self.direction_to_target().map(|direction| -direction)
    }

    fn direction_to_anchor(&self) -> Option<Vec2> {
        let anchor = self.anchor?;
        let to_anchor = anchor.origin() - self.position;
        (to_anchor.length() > anchor.arrival_radius()).then_some(to_anchor.normalize_or_zero())
    }
}

#[derive(Clone, Copy)]
pub(super) enum SteeringEffort {
    Scaled(f32),
    Boosted(f32),
}

pub(super) enum MotionStep {
    Steer {
        direction: Option<Vec2>,
        effort: SteeringEffort,
    },
    /// Ended this frame, maybe leaving a boost to ease off. The brain clears
    /// the slot when the finish flushes, so the next frame's `sync` resets the
    /// executor before it could step the ended behavior again.
    Finished { speed_boost: Option<SpeedBoost> },
}

/// The brain stamps each start with a fresh epoch, so an executor can tell a
/// new run apart from the one it just finished.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunTag {
    epoch: Option<u32>,
}

pub(super) struct MotionContext<'a, 'w> {
    pub input: &'a SteerInput<'w>,
    pub movement: &'a SimpleMotion,
    pub delta_secs: f32,
    pub rng: &'a mut BrainRng,
}

impl MotionContext<'_, '_> {
    pub fn distance_this_frame(&self, speed_boost: f32) -> f32 {
        if self.movement.is_stunned() {
            0.0
        } else {
            self.movement.max_speed * speed_boost * self.movement.speed_scale() * self.delta_secs
        }
    }
}

/// A component that runs one finite [`Motion`]. List new ones in [`FiniteMotions`].
pub(super) trait MotionExecutor: Capability<Key = Motion> {
    fn tag(&mut self) -> &mut RunTag;
    /// Drops any running state; returns a boost when an interrupted burst leaves one.
    fn reset(&mut self, now: f32) -> Option<SpeedBoost>;
    fn step(&mut self, context: &mut MotionContext) -> MotionStep;
}

/// Every finite motion executor an entity may carry. A new one also goes in
/// `finite_executors!`.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct FiniteMotions {
    dash: Option<&'static mut Dash>,
    swoop: Option<&'static mut Swoop>,
}

/// The executors a `FiniteMotions` item carries, as `$as_dyn` trait objects.
macro_rules! finite_executors {
    ($item:expr, $deref:ident, $as_dyn:ty) => {
        [
            $item.dash.$deref().map(|executor| executor as $as_dyn),
            $item.swoop.$deref().map(|executor| executor as $as_dyn),
        ]
        .into_iter()
        .flatten()
    };
}

impl FiniteMotionsItem<'_, '_> {
    fn executors(&mut self) -> impl Iterator<Item = &mut dyn MotionExecutor> {
        finite_executors!(self, as_deref_mut, &mut dyn MotionExecutor)
    }

    fn sync(&mut self, active: Option<ActiveMotion>, now: f32) -> Option<SpeedBoost> {
        let mut speed_boost = None;
        for executor in self.executors() {
            let epoch = active
                .filter(|active| active.motion.index() == executor.key().index())
                .map(|active| active.epoch);
            if executor.tag().epoch != epoch {
                speed_boost = executor.reset(now).or(speed_boost);
                *executor.tag() = RunTag { epoch };
            }
        }
        speed_boost
    }

    fn step(&mut self, motion: Motion, context: &mut MotionContext) -> Option<MotionStep> {
        self.executors()
            .find(|executor| executor.key().index() == motion.index())
            .map(|executor| executor.step(context))
    }
}

impl FiniteMotionsReadOnlyItem<'_, '_> {
    pub(super) fn record_capabilities(&self, capabilities: &mut Capabilities) {
        for executor in finite_executors!(self, as_deref, &dyn MotionExecutor) {
            record_capability(capabilities, Some(executor));
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct SteeredCharacter {
    entity: Entity,
    brain: &'static Brain,
    rng: &'static mut BrainRng,
    movement: &'static mut SimpleMotion,
    transform: &'static Transform,
    target_info: Option<&'static TargetInfo>,
    has_target: Has<Targeting>,
    anchor: Option<&'static Anchor>,
    turn_rate: Option<&'static TurnRate>,
    finite_motions: FiniteMotions,
    speed_boost: Option<&'static SpeedBoost>,
    wander: Option<&'static mut Wander>,
    orbit: Option<&'static mut Orbit>,
}

pub(super) fn steer_characters_from_brain_motion(
    time: Res<Time<Virtual>>,
    mut characters: Query<SteeredCharacter>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    let delta_secs = time.delta_secs();

    for character in &mut characters {
        let SteeredCharacterItem {
            entity,
            brain,
            mut rng,
            mut movement,
            transform,
            target_info,
            has_target,
            anchor,
            turn_rate,
            mut finite_motions,
            speed_boost,
            mut wander,
            mut orbit,
        } = character;

        let active = brain
            .current_motion_behavior()
            .filter(|active| active.motion.is_finite());
        let fresh_boost = finite_motions.sync(active, now);
        // Ahead of any `continue`, so a decayed boost is always removed. A
        // fresh boost from an interrupted burst applies this frame.
        let speed_boost = fresh_boost.or(speed_boost.copied());
        let boost_multiplier = speed_boost.and_then(|boost| boost.compute_speed_multiplier(now));
        if let Some(fresh_boost) = fresh_boost {
            commands.entity(entity).try_insert(fresh_boost);
        } else if speed_boost.is_some() && boost_multiplier.is_none() {
            commands.entity(entity).try_remove::<SpeedBoost>();
        }
        let boost_multiplier = boost_multiplier.unwrap_or(1.0);
        // Ahead of the early `continue`s, so a finite motion between two orbits
        // still makes the next one pick its side from the heading.
        if !matches!(brain.motion(), Some(Motion::Orbit(_)))
            && let Some(orbit) = orbit.as_deref_mut()
        {
            orbit.leave_orbit();
        }

        let missing_target = brain.motion().is_some_and(Motion::needs_target)
            && (!has_target || target_info.is_none());
        if missing_target {
            halt_movement(&mut movement);
            continue;
        }

        let turn_cap = turn_rate.map(|rate| rate.0 * delta_secs);
        let input = SteerInput {
            position: transform.translation.xy(),
            heading: movement.direction,
            now,
            target_info,
            anchor,
        };

        if let Some(active) = active {
            let mut context = MotionContext {
                input: &input,
                movement: &movement,
                delta_secs,
                rng: &mut rng,
            };
            match finite_motions.step(active.motion, &mut context) {
                // A finite motion measures its own distance, so the boost
                // stays out of it.
                Some(MotionStep::Steer { direction, effort }) => {
                    apply_steering_to_movement(&mut movement, direction, effort, 1.0, turn_cap);
                    continue;
                }
                Some(MotionStep::Finished { speed_boost }) => {
                    if let Some(speed_boost) = speed_boost {
                        commands.entity(entity).try_insert(speed_boost);
                    }
                    commands.trigger(MotionFinished {
                        entity,
                        motion: active.motion,
                        epoch: active.epoch,
                    });
                    continue;
                }
                None => {}
            }
        }

        let (direction, effort) = steer_continuous_motion(
            brain.motion(),
            &input,
            MotionMemory {
                wander: wander.as_deref_mut(),
                orbit: orbit.as_deref_mut(),
            },
            &mut rng,
        );
        apply_steering_to_movement(&mut movement, direction, effort, boost_multiplier, turn_cap);
    }
}

struct MotionMemory<'a> {
    wander: Option<&'a mut Wander>,
    orbit: Option<&'a mut Orbit>,
}

fn steer_continuous_motion(
    motion: Option<Motion>,
    input: &SteerInput,
    memory: MotionMemory,
    rng: &mut BrainRng,
) -> (Option<Vec2>, SteeringEffort) {
    let MotionMemory { mut wander, orbit } = memory;
    if motion != Some(Motion::Calm)
        && let Some(wander) = wander.as_deref_mut()
    {
        wander.leave_calm();
    }
    let Some(motion) = motion else {
        return (None, SteeringEffort::Scaled(1.0));
    };
    match motion {
        // Finite motions steer through their executors, never here.
        Motion::Idle | Motion::Dash | Motion::Swoop => (None, SteeringEffort::Scaled(1.0)),
        Motion::Chase => (input.direction_to_target(), SteeringEffort::Scaled(1.0)),
        Motion::Flee => (input.direction_from_target(), SteeringEffort::Scaled(1.0)),
        Motion::ReturnToAnchor => (input.direction_to_anchor(), SteeringEffort::Scaled(1.0)),
        Motion::KeepDistance(distance_range) => (
            steer_to_hold_distance_range(input, distance_range, rng),
            SteeringEffort::Scaled(1.0),
        ),
        Motion::Orbit(distance_range) => {
            let orbit = orbit.expect("orbit is only available with an `Orbit` component");
            (
                orbit::steer(input, orbit, distance_range, rng),
                SteeringEffort::Scaled(1.0),
            )
        }
        Motion::Calm => {
            let wander = wander.expect("calm is only available with a `Wander` component");
            let direction = wander::steer(input, wander, rng);
            (direction, SteeringEffort::Scaled(wander.speed_scale))
        }
    }
}

fn steer_to_hold_distance_range(
    input: &SteerInput,
    distance_range: Interval,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let distance = input.target_info.map_or(0.0, |info| info.distance);
    let Some(direction) = input.direction_to_target() else {
        // Sitting on the target leaves no bearing, so keep the current heading
        // to peel off it.
        if distance < distance_range.min() {
            return Some(
                input
                    .heading
                    .try_normalize()
                    .unwrap_or_else(|| random.direction()),
            );
        }
        return None;
    };
    let error = distance_range.error(distance);
    if error.abs() < KEEP_ARRIVAL_DEADBAND {
        None
    } else if error < 0.0 {
        Some(-direction)
    } else {
        Some(direction)
    }
}

fn halt_movement(movement: &mut SimpleMotion) {
    if !movement.is_idle() {
        movement.stop_moving();
    }
}

fn apply_steering_to_movement(
    movement: &mut SimpleMotion,
    direction: Option<Vec2>,
    effort: SteeringEffort,
    boost_multiplier: f32,
    turn_cap: Option<f32>,
) {
    let Some(desired) = direction else {
        halt_movement(movement);
        return;
    };
    let heading = rotate_heading_toward_desired(movement, desired, turn_cap);
    let speed_multiplier = match effort {
        SteeringEffort::Scaled(scale) => scale.clamp(0.0, 1.0),
        SteeringEffort::Boosted(speed_boost) => speed_boost.max(1.0),
    } * boost_multiplier;
    if speed_multiplier > 1.0 {
        movement.start_moving_boosted(heading, speed_multiplier);
    } else {
        movement.start_moving_scaled(heading, speed_multiplier);
    }
}

fn rotate_heading_toward_desired(
    movement: &SimpleMotion,
    desired: Vec2,
    turn_cap: Option<f32>,
) -> Vec2 {
    if let Some(max_angle) = turn_cap
        && movement.direction.length_squared() > f32::EPSILON
        && desired.length_squared() > f32::EPSILON
    {
        movement.direction.rotate_towards(desired, max_angle)
    } else {
        desired
    }
}
