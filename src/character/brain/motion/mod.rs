//! Per-frame steering. The system that owns the brain's held motion writes the
//! character's [`Steering`], then [`apply_steering`] turns it into [`Kinematics`].

mod anchor;
mod dash;
mod direct;
mod orbit;
mod swoop;
mod wander;

use bevy::{ecs::query::QueryData, prelude::*};

pub use anchor::Anchor;
pub use dash::Dash;
pub use orbit::Orbit;
pub use swoop::Swoop;
pub use wander::Wander;

use super::Brain;
use super::engine::Interval;
use super::perception::Targeting;
use crate::character::{AiSet, state::Kinematics, vision::TargetInfo};
use crate::prelude::sync_slow_from_statuses;

/// How a character moves: a continuous motion that runs until replaced, or a
/// finite committed move that steers alone until its system reports it
/// finished. Add a variant only for a new steering contract.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Motion {
    /// Disengaged: alternates standing and ambling, paced by `Wander`.
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
    /// Circles the target inside the distance range.
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

    /// Position of this motion in per-motion arrays.
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

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            dash::apply_dash,
            swoop::apply_swoop,
            orbit::apply_orbit,
            wander::apply_wander,
            direct::apply_direct_steering,
            apply_steering,
        )
            .chain()
            .in_set(AiSet::Motion)
            .after(sync_slow_from_statuses),
    );
}

/// Caps how fast a character turns, in radians per second. Absent means instant turns.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// Extra speed left over from a finite motion's burst, easing off over
/// `duration`. Multiplies scaled steering until it decays.
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

/// A finite motion completed. The brain starts its cooldown and, if `epoch` is
/// still current, ends its behavior and decides again at once.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
    pub epoch: u32,
}

/// This frame's steering request, written by the system that owns the held
/// motion.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Steering {
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
    /// leftover [`SpeedBoost`].
    pub(super) const fn scaled(direction: Option<Vec2>, scale: f32) -> Self {
        Self {
            direction,
            effort: SteeringEffort::Scaled(scale),
        }
    }

    /// Moves at a fixed `speed_boost` (at least `1.0`) times max speed. A finite
    /// motion measures its own distance at that speed, so leftover boosts stay out.
    pub(super) const fn boosted(direction: Option<Vec2>, speed_boost: f32) -> Self {
        Self {
            direction,
            effort: SteeringEffort::Boosted(speed_boost),
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
        speed_boost: Option<SpeedBoost>,
    },
}

/// Follows which brain run of a finite motion its component is serving, so a
/// new run, or the motion leaving the slot, resets the component's state.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunTag {
    epoch: Option<u32>,
}

impl RunTag {
    /// Catches up with the brain's current run of `motion`; true when that run
    /// differs from the one followed so far.
    pub(super) fn follow_brain_run(&mut self, brain: &Brain, motion: Motion) -> bool {
        let epoch = brain
            .current_motion_behavior()
            .filter(|active| active.motion.index() == motion.index())
            .map(|active| active.epoch);
        let changed = self.epoch != epoch;
        self.epoch = epoch;
        changed
    }

    /// The followed run's epoch while its motion holds the slot.
    pub(super) fn current_epoch(self) -> Option<u32> {
        self.epoch
    }
}

/// Leaves any `speed_boost` to ease off and tells the brain the run is done.
pub(super) fn report_motion_finished(
    commands: &mut Commands,
    entity: Entity,
    motion: Motion,
    epoch: u32,
    speed_boost: Option<SpeedBoost>,
) {
    if let Some(speed_boost) = speed_boost {
        commands.entity(entity).try_insert(speed_boost);
    }
    commands.trigger(MotionFinished {
        entity,
        motion,
        epoch,
    });
}

/// What every motion system reads to steer a character.
#[derive(QueryData)]
pub(super) struct SteeredCharacter {
    brain: &'static Brain,
    transform: &'static Transform,
    kinematics: &'static Kinematics,
    target_info: Option<&'static TargetInfo>,
    has_target: Has<Targeting>,
    anchor: Option<&'static Anchor>,
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
            anchor: self.anchor,
            kinematics: self.kinematics,
        })
    }
}

pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub heading: Vec2,
    pub now: f32,
    pub target_info: Option<&'a TargetInfo>,
    pub anchor: Option<&'a Anchor>,
    delta_secs: f32,
    kinematics: &'a Kinematics,
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
/// fast it turns and dropping a [`SpeedBoost`] once it decays.
fn apply_steering(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut characters: Query<(
        Entity,
        &Steering,
        &mut Kinematics,
        Option<&TurnRate>,
        Option<&SpeedBoost>,
    )>,
) {
    let now = time.elapsed_secs();
    let delta_secs = time.delta_secs();
    for (entity, steering, mut kinematics, turn_rate, speed_boost) in &mut characters {
        let boost_multiplier = speed_boost.and_then(|boost| boost.compute_speed_multiplier(now));
        if speed_boost.is_some() && boost_multiplier.is_none() {
            commands.entity(entity).try_remove::<SpeedBoost>();
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
            SteeringEffort::Boosted(speed_boost) => speed_boost.max(1.0),
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
