use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::RngExt;

use super::{Home, HomeArea, Steering, SteeringMotionOf};
use crate::character::state::{BlockedState, Kinematics};
use crate::prelude::*;

/// The result of advancing a motion by one frame.
pub(super) enum MotionStep {
    Steer(Steering),
    /// The motion ended this frame.
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
    home: Option<&'static Home>,
    blocked: BlockedState,
}

impl SteeredCharacterItem<'_, '_> {
    /// This frame's input for a running motion that steers by `steering_motion_of`, if anything.
    pub(super) fn build_steer_input(
        &self,
        steering_motion_of: Option<&SteeringMotionOf>,
        targets: &SteeringTargets,
        time: &Time<Virtual>,
    ) -> SteerInput<'_> {
        let position = self.transform.translation.xy();
        let target = steering_motion_of
            .and_then(|steering_motion_of| targets.locate(position, steering_motion_of.target));
        let home = self
            .home
            .and_then(|home| targets.locate_home(position, home.get()));

        SteerInput {
            position,
            heading: self.kinematics.direction,
            now: time.elapsed_secs(),
            delta_secs: time.delta_secs(),
            target,
            home,
            kinematics: self.kinematics,
            stat_speed: self.stat_multipliers.speed,
            is_blocked: self.blocked.is_blocked(),
        }
    }
}

/// An entity a motion steers by this frame, relative to the steering character.
#[derive(Clone, Copy, Debug)]
pub(super) struct SteeringTarget {
    pub position: Vec2,
    pub distance: f32,
    pub direction: Vec2,
}

impl SteeringTarget {
    /// The direction to the entity; `None` while the character stands on it.
    pub(super) fn nonzero_direction(&self) -> Option<Vec2> {
        (self.direction != Vec2::ZERO).then_some(self.direction)
    }
}

/// Which way a character circles a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CircleDirection {
    Clockwise,
    CounterClockwise,
}

impl CircleDirection {
    /// The way `heading` already turns around the point that `bearing` points to; random when the
    /// heading has no sideways part.
    pub(super) fn from_heading(bearing: Vec2, heading: Vec2) -> Self {
        let heading_side = bearing.perp().dot(heading);
        if heading_side > f32::EPSILON {
            Self::Clockwise
        } else if heading_side < -f32::EPSILON {
            Self::CounterClockwise
        } else if rand::rng().random_bool(0.5) {
            Self::Clockwise
        } else {
            Self::CounterClockwise
        }
    }

    pub(super) fn reversed(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }

    /// The unit direction that circles, this way, the point that the unit `bearing` points to.
    pub(super) fn find_tangent(self, bearing: Vec2) -> Vec2 {
        match self {
            Self::Clockwise => bearing.perp(),
            Self::CounterClockwise => -bearing.perp(),
        }
    }

    /// The unit direction `angle` radians off the unit `bearing`, turned toward this way's tangent
    /// (see [`Self::find_tangent`]).
    pub(super) fn find_direction_off_bearing(self, bearing: Vec2, angle: f32) -> Vec2 {
        let signed_angle = match self {
            Self::Clockwise => angle,
            Self::CounterClockwise => -angle,
        };
        Rot2::radians(signed_angle) * bearing
    }

    /// `angle` signed to turn this way: negative for clockwise.
    pub(super) fn sign_angle(self, angle: f32) -> f32 {
        match self {
            Self::Clockwise => -angle,
            Self::CounterClockwise => angle,
        }
    }
}

/// A character's [`HomeArea`] this frame, relative to the character.
#[derive(Clone, Copy, Debug)]
pub(super) struct SteeringHome {
    pub center: SteeringTarget,
    pub area: HomeArea,
}

/// Locates the targets and homes of running motions.
#[derive(SystemParam)]
pub(super) struct SteeringTargets<'w, 's> {
    targets: Query<'w, 's, &'static Transform>,
    home_areas: Query<'w, 's, &'static HomeArea>,
}

impl SteeringTargets<'_, '_> {
    /// `target` relative to `observer_position`; `None` if the target no longer exists.
    fn locate(&self, observer_position: Vec2, target: Entity) -> Option<SteeringTarget> {
        let position = self.targets.get(target).ok()?.translation.xy();
        let offset = position - observer_position;
        Some(SteeringTarget {
            position,
            distance: offset.length(),
            direction: offset.normalize_or_zero(),
        })
    }

    /// The `home` entity's [`HomeArea`] relative to `observer_position`; `None` if it no longer
    /// exists.
    fn locate_home(&self, observer_position: Vec2, home: Entity) -> Option<SteeringHome> {
        Some(SteeringHome {
            center: self.locate(observer_position, home)?,
            area: *self.home_areas.get(home).ok()?,
        })
    }
}

pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub heading: Vec2,
    pub now: f32,
    pub target: Option<SteeringTarget>,
    pub home: Option<SteeringHome>,
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
        let home = self.home?;
        if home.area.has_arrived(home.center.distance) {
            return None;
        }
        home.center.nonzero_direction()
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
