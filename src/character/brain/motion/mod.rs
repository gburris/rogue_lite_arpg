//! Per-frame steering: turns the brain's chosen motion into [`SimpleMotion`].

mod dash;
mod orbit;
mod swoop;
mod wander;

use bevy::{ecs::query::QueryData, prelude::*};

pub use dash::Dash;
pub use orbit::Orbit;
pub use swoop::Swoop;
pub use wander::Wander;

use super::capability::{Capabilities, Capability, record_capability};
use super::{ActiveMotion, Brain, BrainRng, Interval, Motion};
use crate::character::{
    anchor::Anchor,
    state::SimpleMotion,
    vision::{TargetInfo, Targeting},
};
use crate::prelude::TILE_SIZE;

pub(crate) const ANCHOR_ARRIVAL_RADIUS: f32 = TILE_SIZE as f32 / 2.0;
const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Caps how fast a character turns, in radians per second. Absent means instant turns.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// Which way a character circles its target. Motions share it so one can turn
/// the orbit around without knowing the orbit motion exists.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct OrbitDirection {
    clockwise: Option<bool>,
    held_until: f32,
}

impl OrbitDirection {
    /// Multiplier on `bearing.perp()` for this direction's tangent. Picks a
    /// direction at random the first time it is read.
    pub(super) fn sign(&mut self, random: &mut BrainRng) -> f32 {
        let clockwise = *self
            .clockwise
            .get_or_insert_with(|| random_clockwise(random));
        if clockwise { -1.0 } else { 1.0 }
    }

    pub(super) fn flip(&mut self, random: &mut BrainRng) {
        let clockwise = self
            .clockwise
            .map_or_else(|| random_clockwise(random), |clockwise| !clockwise);
        self.clockwise = Some(clockwise);
    }

    pub(super) fn reroll(&mut self, switch_chance: f32, random: &mut BrainRng) {
        if self.clockwise.is_none() || random.range(0.0, 1.0) < switch_chance {
            self.flip(random);
        }
    }

    pub(super) fn hold_until(&mut self, until: f32) {
        self.held_until = self.held_until.max(until);
    }

    pub(super) fn is_held(&self, now: f32) -> bool {
        now < self.held_until
    }
}

fn random_clockwise(random: &mut BrainRng) -> bool {
    random.range(0.0, 1.0) < 0.5
}

/// A finite motion completed; the brain starts its cooldown and decides again.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
}

/// A finite motion could not start; the brain drops it without a cooldown.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionAborted {
    pub entity: Entity,
    pub motion: Motion,
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
        let to_anchor = self.anchor?.origin() - self.position;
        (to_anchor.length() > ANCHOR_ARRIVAL_RADIUS).then_some(to_anchor.normalize_or_zero())
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
    /// Ended this frame; the brain has not been told yet.
    Finished,
    Aborted,
    /// Ended on an earlier frame; waiting for the brain to release it.
    Spent,
}

/// The brain stamps each start with a fresh epoch, so an executor can tell a
/// new run apart from the one it just finished.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunTag {
    epoch: Option<u32>,
    spent: bool,
}

pub(super) struct MotionContext<'a, 'w> {
    pub input: &'a SteerInput<'w>,
    pub movement: &'a SimpleMotion,
    pub delta_secs: f32,
    pub rng: &'a mut BrainRng,
    pub orbit_direction: Option<&'a mut OrbitDirection>,
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
    fn reset(&mut self, now: f32);
    fn step(&mut self, context: &mut MotionContext) -> MotionStep;
    fn recovery_boost(&self, _now: f32) -> f32 {
        1.0
    }
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

    fn sync(&mut self, active: Option<ActiveMotion>, now: f32) {
        for executor in self.executors() {
            let epoch = active
                .filter(|active| active.motion.index() == executor.key().index())
                .map(|active| active.epoch);
            if executor.tag().epoch != epoch {
                executor.reset(now);
                *executor.tag() = RunTag {
                    epoch,
                    spent: false,
                };
            }
        }
    }

    fn step(&mut self, motion: Motion, context: &mut MotionContext) -> Option<MotionStep> {
        self.executors()
            .find(|executor| executor.key().index() == motion.index())
            .map(|executor| executor.step(context))
    }

    fn recovery_boost(&mut self, now: f32) -> f32 {
        self.executors()
            .map(|executor| executor.recovery_boost(now))
            .fold(1.0, f32::max)
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
    wander: Option<&'static mut Wander>,
    orbit: Option<&'static mut Orbit>,
    orbit_direction: Option<&'static mut OrbitDirection>,
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
            mut wander,
            mut orbit,
            mut orbit_direction,
        } = character;

        let active = brain.active_motion();
        finite_motions.sync(active, now);

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
                orbit_direction: orbit_direction.as_deref_mut(),
            };
            match finite_motions.step(active.motion, &mut context) {
                Some(MotionStep::Steer { direction, effort }) => {
                    apply_steering_to_movement(&mut movement, direction, effort, turn_cap);
                    continue;
                }
                Some(MotionStep::Finished) => {
                    commands.trigger(MotionFinished {
                        entity,
                        motion: active.motion,
                    });
                    continue;
                }
                Some(MotionStep::Aborted) => {
                    commands.trigger(MotionAborted {
                        entity,
                        motion: active.motion,
                    });
                    continue;
                }
                Some(MotionStep::Spent) | None => {}
            }
        }

        let recovery_boost = finite_motions.recovery_boost(now);
        let (direction, effort) = steer_continuous_motion(
            brain.motion(),
            &input,
            MotionMemory {
                wander: wander.as_deref_mut(),
                orbit: orbit.as_deref_mut(),
                orbit_direction: orbit_direction.as_deref_mut(),
                recovery_boost,
            },
            &mut rng,
        );
        apply_steering_to_movement(&mut movement, direction, effort, turn_cap);
    }
}

struct MotionMemory<'a> {
    wander: Option<&'a mut Wander>,
    orbit: Option<&'a mut Orbit>,
    orbit_direction: Option<&'a mut OrbitDirection>,
    recovery_boost: f32,
}

fn steer_continuous_motion(
    motion: Option<Motion>,
    input: &SteerInput,
    memory: MotionMemory,
    rng: &mut BrainRng,
) -> (Option<Vec2>, SteeringEffort) {
    let MotionMemory {
        mut wander,
        orbit,
        orbit_direction,
        recovery_boost,
    } = memory;
    if motion != Some(Motion::Calm)
        && let Some(wander) = wander.as_deref_mut()
    {
        wander.leave_calm();
    }
    let Some(motion) = motion else {
        return (None, SteeringEffort::Scaled(1.0));
    };
    match motion {
        Motion::Idle => (None, SteeringEffort::Scaled(1.0)),
        Motion::Chase => (input.direction_to_target(), SteeringEffort::Scaled(1.0)),
        Motion::Flee => (input.direction_from_target(), SteeringEffort::Scaled(1.0)),
        Motion::ReturnToAnchor => (input.direction_to_anchor(), SteeringEffort::Scaled(1.0)),
        Motion::KeepDistance(distance_range) => (
            steer_to_hold_distance_range(input, distance_range, rng),
            SteeringEffort::Boosted(recovery_boost),
        ),
        Motion::Orbit(distance_range) => (
            orbit::steer(input, orbit, orbit_direction, distance_range, rng),
            SteeringEffort::Boosted(recovery_boost),
        ),
        Motion::Calm => {
            let direction = wander::steer(input, wander.as_deref_mut(), rng);
            let scale = wander::speed_scale(wander.as_deref());
            (direction, SteeringEffort::Scaled(scale))
        }
        Motion::Dash | Motion::Swoop => (None, SteeringEffort::Scaled(1.0)),
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
    turn_cap: Option<f32>,
) {
    let Some(desired) = direction else {
        halt_movement(movement);
        return;
    };
    let heading = rotate_heading_toward_desired(movement, desired, turn_cap);
    match effort {
        SteeringEffort::Scaled(scale) => movement.start_moving_scaled(heading, scale),
        SteeringEffort::Boosted(speed_boost) => movement.start_moving_boosted(heading, speed_boost),
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
