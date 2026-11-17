//! One frame of a motion: what its system reads to steer, and how a finite
//! motion steps.

use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use super::{HomeArea, Motion, Steering};
use crate::character::brain::{TargetInfo, Targeting};
use crate::character::state::{BlockedState, Kinematics};
use crate::prelude::*;

/// One frame of a finite motion.
pub(super) enum MotionStep {
    Steer(Steering),
    /// Ended this frame; its system removes the run's progress component.
    Finished,
}

/// How many times faster than it fills a [`BlockedTimer`] empties while the character moves
/// freely.
const BLOCKED_DRAIN_RATE: f32 = 4.0;

/// How long a motion has pushed against a wall or a crowd without getting anywhere.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BlockedTimer {
    seconds: f32,
}

impl BlockedTimer {
    /// Counts up while the character `is_blocked` and back down while it moves freely. True,
    /// restarting the count, once it reaches `limit` seconds.
    pub(super) fn tick_and_check_limit(
        &mut self,
        is_blocked: bool,
        delta_secs: f32,
        limit: f32,
    ) -> bool {
        if !is_blocked {
            // Drained rather than cleared: a blocked character still reads as free on the odd
            // frame, which must not restart the count.
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

    /// Lets the character stay blocked `seconds` longer before the limit counts, such as while
    /// it turns away from what blocked it.
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
    target_info: Option<&'static TargetInfo>,
    has_target: Has<Targeting>,
    home_area: Option<&'static HomeArea>,
    blocked: BlockedState,
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
            stat_speed: self.stat_multipliers.speed,
            is_blocked: self.blocked.is_blocked(),
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
    stat_speed: f32,
    is_blocked: bool,
}

impl SteerInput<'_> {
    pub(super) fn direction_to_target(&self) -> Option<Vec2> {
        self.target_info.and_then(TargetInfo::nonzero_direction)
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

    /// How far the character covers this frame at `speed_multiplier` times its max speed, after
    /// its [`StatMultipliers`] speed.
    pub(super) fn distance_this_frame(&self, speed_multiplier: f32) -> f32 {
        self.kinematics.max_speed * self.stat_speed * speed_multiplier * self.delta_secs
    }
}
