//! Per-frame steering: the [`Steering`] request each motion system writes, and
//! [`apply_steering`], which turns it into [`Kinematics`].

use std::time::Duration;

use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use super::{HomeArea, Motion};
use crate::character::brain::{Brain, Targeting};
use crate::character::state::Kinematics;
use crate::character::vision::TargetInfo;

/// Caps how fast a character turns, in radians per second. Absent means instant turns.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// Extra speed left over from a finite motion's burst, easing off over
/// `duration`. Multiplies scaled steering until it decays.
#[derive(Component, Clone, Debug)]
pub(super) struct DecayingSpeedBoost {
    extra_speed: f32,
    timer: Timer,
    easing: EaseFunction,
}

impl DecayingSpeedBoost {
    pub(super) fn new(extra_speed: f32, duration: f32, easing: EaseFunction) -> Self {
        Self {
            extra_speed,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            easing,
        }
    }

    /// Advances the boost by `delta` and returns its speed multiplier, or
    /// `None` once it has decayed.
    fn tick_and_compute_speed_multiplier(&mut self, delta: Duration) -> Option<f32> {
        self.timer.tick(delta);
        if self.timer.is_finished() {
            return None;
        }
        Some(1.0 + self.extra_speed * self.easing.sample_clamped(self.timer.fraction_remaining()))
    }
}

/// A finite motion completed. The brain starts its cooldown and, if `behavior_run_id` is
/// still current, ends its behavior and decides again at once.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
    pub behavior_run_id: u32,
}

/// This frame's steering request, written by the system that owns the held
/// motion.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::character::brain) struct Steering {
    direction: Option<Vec2>,
    effort: SteeringEffort,
}

impl Default for Steering {
    fn default() -> Self {
        Self::HALT
    }
}

impl Steering {
    pub(super) const HALT: Self = Self::scaled(None, 1.0);

    /// Moves at `scale` (clamped to `0..=1`) times max speed, raised by any
    /// leftover [`DecayingSpeedBoost`].
    pub(super) const fn scaled(direction: Option<Vec2>, scale: f32) -> Self {
        Self {
            direction,
            effort: SteeringEffort::Scaled(scale),
        }
    }

    /// Moves at a fixed `speed_multiplier` (at least `1.0`) times max speed. A finite
    /// motion measures its own distance at that speed, so leftover boosts stay out.
    pub(super) const fn boosted(direction: Option<Vec2>, speed_multiplier: f32) -> Self {
        Self {
            direction,
            effort: SteeringEffort::Boosted(speed_multiplier),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SteeringEffort {
    Scaled(f32),
    Boosted(f32),
}

/// One frame of a finite motion.
pub(super) enum MotionStep {
    Steer(Steering),
    /// Ended this frame, maybe leaving a boost to ease off. The brain clears
    /// the slot when the finish flushes, so the next frame's run check resets
    /// the component before it could step the ended run again.
    Finished {
        decaying_speed_boost: Option<DecayingSpeedBoost>,
    },
}

/// The behavior run id a finite motion's component is serving, so a new run,
/// or the motion leaving the slot, resets the component's state.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CurrentBehaviorRunId {
    behavior_run_id: Option<u32>,
}

impl CurrentBehaviorRunId {
    /// Catches up with the brain's current run of `motion`; true when that run
    /// differs from the one followed so far.
    pub(super) fn follow_brain_run(&mut self, brain: &Brain, motion: Motion) -> bool {
        let behavior_run_id = brain
            .current_motion_behavior()
            .filter(|running| {
                running
                    .motion
                    .is_some_and(|running_motion| running_motion.index() == motion.index())
            })
            .map(|running| running.behavior_run_id);
        let changed = self.behavior_run_id != behavior_run_id;
        self.behavior_run_id = behavior_run_id;
        changed
    }

    /// The id of the followed run while its motion holds the slot.
    pub(super) fn get(self) -> Option<u32> {
        self.behavior_run_id
    }
}

/// Leaves any `decaying_speed_boost` to ease off and tells the brain the run is done.
pub(super) fn report_motion_finished(
    commands: &mut Commands,
    entity: Entity,
    motion: Motion,
    behavior_run_id: u32,
    decaying_speed_boost: Option<DecayingSpeedBoost>,
) {
    if let Some(decaying_speed_boost) = decaying_speed_boost {
        commands.entity(entity).try_insert(decaying_speed_boost);
    }
    commands.trigger(MotionFinished {
        entity,
        motion,
        behavior_run_id,
    });
}

/// What every motion system reads to steer a character.
#[derive(QueryData)]
pub(super) struct SteeredCharacter {
    pub(super) brain: &'static Brain,
    transform: &'static Transform,
    kinematics: &'static Kinematics,
    target_info: Option<&'static TargetInfo>,
    has_target: Has<Targeting>,
    home_area: Option<&'static HomeArea>,
}

impl SteeredCharacterItem<'_, '_> {
    /// This frame's input for steering `motion`, or `None` while it needs a
    /// target the character lacks.
    pub(super) fn build_steer_input(
        &self,
        motion: Motion,
        time: &Time<Virtual>,
    ) -> Option<SteerInput<'_>> {
        if motion.needs_target() && (!self.has_target || self.target_info.is_none()) {
            return None;
        }
        Some(SteerInput {
            position: self.transform.translation.xy(),
            heading: self.kinematics.direction,
            now: time.elapsed_secs(),
            delta_secs: time.delta_secs(),
            target_info: self.target_info,
            home_area: self.home_area,
            kinematics: self.kinematics,
        })
    }
}

pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub heading: Vec2,
    pub now: f32,
    pub target_info: Option<&'a TargetInfo>,
    pub home_area: Option<&'a HomeArea>,
    delta_secs: f32,
    kinematics: &'a Kinematics,
}

impl SteerInput<'_> {
    pub(super) fn direction_to_target(&self) -> Option<Vec2> {
        let direction = self.target_info.map_or(Vec2::ZERO, |info| info.direction);
        (direction.length_squared() > f32::EPSILON).then_some(direction)
    }

    pub(super) fn direction_from_target(&self) -> Option<Vec2> {
        self.direction_to_target().map(|direction| -direction)
    }

    pub(super) fn direction_to_home(&self) -> Option<Vec2> {
        let home_area = self.home_area?;
        let to_home = home_area.origin() - self.position;
        (to_home.length() > home_area.arrival_radius()).then_some(to_home.normalize_or_zero())
    }

    /// How far the character covers this frame at `speed_multiplier` times
    /// max speed, after any slow.
    pub(super) fn distance_this_frame(&self, speed_multiplier: f32) -> f32 {
        self.kinematics.max_speed
            * speed_multiplier
            * self.kinematics.speed_scale()
            * self.delta_secs
    }
}

/// Turns each brain character's [`Steering`] into [`Kinematics`], capping how
/// fast it turns and dropping a [`DecayingSpeedBoost`] once it decays.
pub(super) fn apply_steering(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut characters: Query<(
        Entity,
        &Steering,
        &mut Kinematics,
        Option<&TurnRate>,
        Option<&mut DecayingSpeedBoost>,
    )>,
) {
    let delta_secs = time.delta_secs();
    for (entity, steering, mut kinematics, turn_rate, mut decaying_speed_boost) in &mut characters {
        let boost_multiplier = decaying_speed_boost
            .as_mut()
            .and_then(|boost| boost.tick_and_compute_speed_multiplier(time.delta()));
        if decaying_speed_boost.is_some() && boost_multiplier.is_none() {
            commands.entity(entity).try_remove::<DecayingSpeedBoost>();
        }
        let Some(desired) = steering.direction else {
            if !kinematics.is_idle() {
                kinematics.stop_moving();
            }
            continue;
        };
        let speed_multiplier = match steering.effort {
            SteeringEffort::Scaled(scale) => {
                scale.clamp(0.0, 1.0) * boost_multiplier.unwrap_or(1.0)
            }
            SteeringEffort::Boosted(speed_multiplier) => speed_multiplier.max(1.0),
        };
        let turn_cap = turn_rate.map(|rate| rate.0 * delta_secs);
        let heading = rotate_heading_toward_desired(kinematics.direction, desired, turn_cap);
        kinematics.move_in_direction(heading, speed_multiplier);
    }
}

fn rotate_heading_toward_desired(heading: Vec2, desired: Vec2, turn_cap: Option<f32>) -> Vec2 {
    if let Some(max_angle) = turn_cap
        && heading.length_squared() > f32::EPSILON
        && desired.length_squared() > f32::EPSILON
    {
        heading.rotate_towards(desired, max_angle)
    } else {
        desired
    }
}
