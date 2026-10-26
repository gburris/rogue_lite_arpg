//! Per-frame steering execution: translates [`Brain`](super::Brain) motions into
//! [`SimpleMotion`].
//!
//! [`super`] decides at ~10 Hz what to do; this module drives every frame how
//! to move. Each behavior owns its component, its memory and its steering in
//! one file: [`strafe`], [`wander`], [`dash`] and [`swoop`]. The
//! stateless steering lives here. The memories are opt-in capabilities that
//! persist across motion switches, so they stay separate components rather than
//! one enum.
//!
//! A running finite motion takes exclusive control of steering. Finite motion
//! components implement [`MotionExecutor`] and are listed once, in
//! [`FiniteMotions`]; the brain's active motion is the single source of truth,
//! and an executor reports back only by triggering [`MotionFinished`] or
//! [`MotionAborted`], never by
//! writing decision state.

mod dash;
mod strafe;
mod swoop;
mod wander;

use bevy::{ecs::query::QueryData, prelude::*};

pub use dash::Dash;
pub use strafe::Strafe;
pub use swoop::Swoop;
pub use wander::Wander;

use super::{ActiveMotion, Brain, BrainRng, Interval, Motion, Tuning};
use crate::character::{
    anchor::Anchor,
    state::SimpleMotion,
    vision::{TargetInfo, Targeting},
};
use crate::prelude::TILE_SIZE;

pub(crate) const ANCHOR_ARRIVAL_RADIUS: f32 = TILE_SIZE as f32 / 2.0;
const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Maximum turn rate in radians per second. Absent means snap.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// A finite motion ran to its end. The brain releases it, starts its cooldown
/// and decides again on the spot.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
}

/// A finite motion could not start. The brain drops it without a cooldown and
/// decides again on the spot.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionAborted {
    pub entity: Entity,
    pub motion: Motion,
}

/// Read-only world input shared by every steering behavior this frame.
pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub now: f32,
    pub target_info: Option<&'a TargetInfo>,
    pub anchor: Option<&'a Anchor>,
}

impl SteerInput<'_> {
    /// Straight at the target, when it has a usable bearing.
    pub(super) fn direction_to_target(&self) -> Option<Vec2> {
        let direction = self.target_info.map_or(Vec2::ZERO, |info| info.direction);
        (direction.length_squared() > f32::EPSILON).then_some(direction)
    }

    /// Straight away from the target.
    fn direction_from_target(&self) -> Option<Vec2> {
        self.direction_to_target().map(|direction| -direction)
    }

    /// Toward the anchor until the arrival radius.
    fn direction_to_anchor(&self) -> Option<Vec2> {
        let to_anchor = self.anchor?.origin() - self.position;
        (to_anchor.length() > ANCHOR_ARRIVAL_RADIUS).then_some(to_anchor.normalize_or_zero())
    }
}

/// How hard to push along the steered heading: a fraction of max speed for
/// continuous motions, a multiple for committed bursts.
#[derive(Clone, Copy)]
pub(super) enum SteeringEffort {
    Scaled(f32),
    Boosted(f32),
}

/// One frame of a finite motion executor.
pub(super) enum MotionStep {
    /// Still running: steer this way.
    Steer {
        direction: Option<Vec2>,
        effort: SteeringEffort,
    },
    /// Ended this frame; the brain has not heard yet.
    Finished,
    /// Could not start; the brain has not heard yet.
    Aborted,
    /// Ended earlier and is waiting for the brain to release it.
    Spent,
}

/// Which brain-issued run an executor is carrying out. The brain stamps every
/// start with a fresh epoch, so an executor can tell a new run of the same
/// motion from the one it just finished, even with no frame in between.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunTag {
    epoch: Option<u32>,
    /// The run ended and the brain has not yet released it.
    spent: bool,
}

/// Everything a finite motion step may look at this frame. The random stream is
/// the only thing it can mutate.
pub(super) struct MotionContext<'a, 'w> {
    pub input: &'a SteerInput<'w>,
    pub movement: &'a SimpleMotion,
    pub delta_secs: f32,
    pub rng: &'a mut BrainRng,
}

impl MotionContext<'_, '_> {
    /// World units one straight-run steering step may consume this frame.
    /// Stunned bodies hold still so committed passes neither advance nor finish.
    pub fn distance_this_frame(&self, speed_boost: f32) -> f32 {
        if self.movement.is_stunned() {
            0.0
        } else {
            self.movement.max_speed * speed_boost * self.movement.speed_scale() * self.delta_secs
        }
    }
}

/// A capability component that carries out one finite [`Motion`]. Implementing this
/// and listing the component in [`FiniteMotions`] is all the executor side of a
/// new finite motion needs.
pub(super) trait MotionExecutor {
    fn motion(&self) -> Motion;
    /// Chance, spacing and reach the decision tick applies to this motion.
    fn tuning(&self) -> Tuning;
    fn tag(&mut self) -> &mut RunTag;
    /// Forgets a run the brain is no longer asking for.
    fn reset(&mut self, now: f32);
    fn step(&mut self, context: &mut MotionContext) -> MotionStep;
    /// Extra speed steering should keep while settling out of the motion.
    fn recovery_boost(&self, _now: f32) -> f32 {
        1.0
    }
}

/// Every finite motion executor an entity may carry. The one place to list a new one.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct FiniteMotions {
    dash: Option<&'static mut Dash>,
    swoop: Option<&'static mut Swoop>,
}

impl FiniteMotionsItem<'_, '_> {
    fn executors(&mut self) -> impl Iterator<Item = &mut dyn MotionExecutor> {
        [
            self.dash
                .as_deref_mut()
                .map(|executor| executor as &mut dyn MotionExecutor),
            self.swoop
                .as_deref_mut()
                .map(|executor| executor as &mut dyn MotionExecutor),
        ]
        .into_iter()
        .flatten()
    }

    /// Resets any executor holding a run the brain no longer asks for, so the
    /// brain's active motion stays the single source of truth.
    fn sync(&mut self, active: Option<ActiveMotion>, now: f32) {
        for executor in self.executors() {
            let epoch = active
                .filter(|active| active.motion.index() == executor.motion().index())
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

    /// Runs the executor for `motion`, if this entity has one.
    fn step(&mut self, motion: Motion, context: &mut MotionContext) -> Option<MotionStep> {
        self.executors()
            .find(|executor| executor.motion().index() == motion.index())
            .map(|executor| executor.step(context))
    }

    /// Largest recovery boost any executor still holds.
    fn recovery_boost(&mut self, now: f32) -> f32 {
        self.executors()
            .map(|executor| executor.recovery_boost(now))
            .fold(1.0, f32::max)
    }
}

impl FiniteMotionsReadOnlyItem<'_, '_> {
    /// Tuning of every finite motion this entity can carry out, by motion index.
    pub(super) fn tuning(&self) -> [Option<Tuning>; Motion::COUNT] {
        let executors: [Option<&dyn MotionExecutor>; 2] = [
            self.dash.map(|executor| executor as &dyn MotionExecutor),
            self.swoop.map(|executor| executor as &dyn MotionExecutor),
        ];
        let mut tuning = [None; Motion::COUNT];
        for executor in executors.into_iter().flatten() {
            tuning[executor.motion().index()] = Some(executor.tuning());
        }
        tuning
    }
}

/// Everything one steered character needs this frame.
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
    strafe: Option<&'static mut Strafe>,
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
            mut strafe,
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
                Some(MotionStep::Steer { direction, effort }) => {
                    apply_steering_to_movement(&mut movement, direction, effort, turn_cap);
                    continue;
                }
                // Fall through to steering this frame so the body never
                // coasts for a frame between a finite motion and its settle-out.
                // Hold the last steer for this frame: the brain decides again at
                // the next sync point, so the body never stops for a frame
                // between a locked motion and the behavior that follows.
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
                strafe: strafe.as_deref_mut(),
                recovery_boost,
            },
            &mut rng,
        );
        apply_steering_to_movement(&mut movement, direction, effort, turn_cap);
    }
}

/// The opt-in memories and tuning continuous steering may use.
struct MotionMemory<'a> {
    wander: Option<&'a mut Wander>,
    strafe: Option<&'a mut Strafe>,
    /// Extra speed held while settling out of a finite motion; also freezes the
    /// orbit side so re-entering the circle does not flip the tangent.
    recovery_boost: f32,
}

/// Where and how hard the current continuous motion wants to push.
fn steer_continuous_motion(
    motion: Option<Motion>,
    input: &SteerInput,
    memory: MotionMemory,
    rng: &mut BrainRng,
) -> (Option<Vec2>, SteeringEffort) {
    let MotionMemory {
        mut wander,
        strafe,
        recovery_boost,
    } = memory;
    if motion != Some(Motion::Calm)
        && let Some(wander) = wander.as_deref_mut()
    {
        wander.leave_calm();
    }
    // An empty slot stands still, the same as an idle motion.
    let Some(motion) = motion else {
        return (None, SteeringEffort::Scaled(1.0));
    };
    match motion {
        Motion::Idle => (None, SteeringEffort::Scaled(1.0)),
        Motion::Chase => (input.direction_to_target(), SteeringEffort::Scaled(1.0)),
        // Still in the fight: run from the target, not toward home.
        // Going home happens only once the target is lost.
        Motion::Flee => (input.direction_from_target(), SteeringEffort::Scaled(1.0)),
        Motion::ReturnToAnchor => (input.direction_to_anchor(), SteeringEffort::Scaled(1.0)),
        Motion::KeepDistance(distance_range) => (
            steer_to_hold_distance_range(input, distance_range, wander, rng),
            SteeringEffort::Boosted(recovery_boost),
        ),
        Motion::Strafe(distance_range) => (
            strafe::steer(input, strafe, distance_range, rng, recovery_boost > 1.0),
            SteeringEffort::Boosted(recovery_boost),
        ),
        Motion::Calm => {
            let direction = wander::steer(input, wander.as_deref_mut(), rng);
            let scale = wander::speed_scale(wander.as_deref());
            (direction, SteeringEffort::Scaled(scale))
        }
        // Finite motions steer through their executors, never as the continuous motion.
        Motion::Dash | Motion::Swoop => (None, SteeringEffort::Scaled(1.0)),
    }
}

/// Holds the band; a small deadband keeps the arrival from flickering.
fn steer_to_hold_distance_range(
    input: &SteerInput,
    distance_range: Interval,
    wander: Option<&mut Wander>,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let distance = input.target_info.map_or(0.0, |info| info.distance);
    let Some(direction) = input.direction_to_target() else {
        if distance < distance_range.min() {
            return Some(wander?.jitter(input.now, random));
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
