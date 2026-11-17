use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{HomeArea, RunningMotion, Steering};
use crate::character::state::{BlockedState, Kinematics};
use crate::prelude::*;

/// The result of advancing a finite motion by one frame.
pub(super) enum MotionStep {
    Steer(Steering),
    /// The motion ended this frame; its system removes the progress component.
    Finished,
}

/// How much faster a [`BlockedTimer`] counts down while unblocked than it counts up while
/// blocked.
const BLOCKED_DRAIN_RATE: f32 = 4.0;

/// How long a character has been blocked; see [`BlockedState`].
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BlockedTimer {
    seconds: f32,
}

impl BlockedTimer {
    /// Counts up while `is_blocked` and down otherwise. Returns true and resets once the count
    /// reaches `limit` seconds.
    pub(super) fn tick_and_check_limit(
        &mut self,
        is_blocked: bool,
        delta_secs: f32,
        limit: f32,
    ) -> bool {
        if !is_blocked {
            // Count down instead of resetting: a blocked character can read as unblocked for
            // single frames.
            self.seconds = (self.seconds - BLOCKED_DRAIN_RATE * delta_secs).max(0.0);
            return false;
        }

        self.seconds += delta_secs;
        if self.seconds < limit {
            return false;
        }
        self.seconds = 0.0;
        true
    }

    pub(super) fn reset(&mut self) {
        self.seconds = 0.0;
    }

    /// Allows `seconds` more of being blocked before the limit is reached.
    pub(super) fn delay_by(&mut self, seconds: f32) {
        self.seconds = -seconds;
    }
}

/// What every motion system reads to steer a character.
#[derive(QueryData)]
pub(super) struct SteeredCharacter {
    transform: &'static Transform,
    kinematics: &'static Kinematics,
    stat_multipliers: &'static StatMultipliers,
    home_area: Option<&'static HomeArea>,
    blocked: BlockedState,
}

impl SteeredCharacterItem<'_, '_> {
    /// This frame's input for steering `running`; `None` if it needs a target and the target is
    /// missing.
    pub(super) fn build_steer_input(
        &self,
        running: &RunningMotion,
        targets: &SteeringTargets,
        time: &Time<Virtual>,
    ) -> Option<SteerInput<'_>> {
        let position = self.transform.translation.xy();
        let target = running
            .target
            .and_then(|target| targets.locate(position, target));
        if running.motion.needs_target() && target.is_none() {
            return None;
        }

        Some(SteerInput {
            position,
            heading: self.kinematics.direction,
            now: time.elapsed_secs(),
            delta_secs: time.delta_secs(),
            target,
            home_area: self.home_area,
            kinematics: self.kinematics,
            stat_speed: self.stat_multipliers.speed,
            is_blocked: self.blocked.is_blocked(),
        })
    }
}

/// A motion's target this frame, relative to the steering character.
#[derive(Clone, Copy, Debug)]
pub(super) struct SteeringTarget {
    pub position: Vec2,
    pub distance: f32,
    pub direction: Vec2,
    pub velocity: Vec2,
}

impl SteeringTarget {
    /// The direction to the target; `None` while the characters overlap.
    fn nonzero_direction(&self) -> Option<Vec2> {
        (self.direction.length_squared() > f32::EPSILON).then_some(self.direction)
    }
}

/// Locates the targets of running motions.
#[derive(SystemParam)]
pub(super) struct SteeringTargets<'w, 's> {
    targets: Query<
        'w,
        's,
        (
            &'static Transform,
            Option<(&'static Kinematics, &'static StatMultipliers)>,
        ),
    >,
}

impl SteeringTargets<'_, '_> {
    /// `target` relative to `observer_position`; `None` if the target no longer exists.
    fn locate(&self, observer_position: Vec2, target: Entity) -> Option<SteeringTarget> {
        let (transform, movement) = self.targets.get(target).ok()?;
        let position = transform.translation.xy();
        let velocity = movement.map_or(Vec2::ZERO, |(kinematics, stat_multipliers)| {
            kinematics.velocity(stat_multipliers.speed)
        });

        Some(SteeringTarget {
            position,
            distance: position.distance(observer_position),
            direction: (position - observer_position).normalize_or_zero(),
            velocity,
        })
    }
}

pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub heading: Vec2,
    pub now: f32,
    pub target: Option<SteeringTarget>,
    pub home_area: Option<&'a HomeArea>,
    delta_secs: f32,
    kinematics: &'a Kinematics,
    stat_speed: f32,
    is_blocked: bool,
}

impl SteerInput<'_> {
    pub(super) fn direction_to_target(&self) -> Option<Vec2> {
        self.target
            .as_ref()
            .and_then(SteeringTarget::nonzero_direction)
    }

    pub(super) fn direction_from_target(&self) -> Option<Vec2> {
        self.direction_to_target().map(|direction| -direction)
    }

    pub(super) fn direction_to_home(&self) -> Option<Vec2> {
        let home_area = self.home_area?;
        if home_area.has_arrived(self.position) {
            return None;
        }
        home_area.direction_home(self.position)
    }

    /// Advances `blocked_timer` by this frame; see [`BlockedTimer::tick_and_check_limit`] and
    /// [`BlockedState`].
    pub(super) fn tick_blocked_timer(&self, blocked_timer: &mut BlockedTimer, limit: f32) -> bool {
        blocked_timer.tick_and_check_limit(self.is_blocked, self.delta_secs, limit)
    }

    /// Distance covered this frame at `speed_multiplier` times max speed, scaled by the
    /// [`StatMultipliers`] speed.
    pub(super) fn distance_this_frame(&self, speed_multiplier: f32) -> f32 {
        self.kinematics.max_speed * self.stat_speed * speed_multiplier * self.delta_secs
    }
}
