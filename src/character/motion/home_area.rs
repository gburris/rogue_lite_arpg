use bevy::prelude::*;

use super::{BlockedTimer, CurrentMotion, Motion, RunningMotion};
use crate::character::state::BlockedState;

/// Seconds a return home may stay blocked before the home moves to where the character stands.
const RETURN_BLOCKED_SECONDS: f32 = 1.0;

/// A character's home: its center, how far it may wander from it, and how close to the center
/// counts as arrived. If the character is blocked on the way back, its home moves to where it
/// stands.
#[derive(Component, Clone)]
pub struct HomeArea {
    origin: Vec2,
    radius: f32,
    arrival_radius: f32,
    return_blocked_timer: BlockedTimer,
}

impl HomeArea {
    pub fn new(origin: Vec2, radius: f32, arrival_radius: f32) -> Self {
        debug_assert!(
            radius.is_finite() && radius > 0.0,
            "home area radius must be finite and positive"
        );
        debug_assert!(
            arrival_radius > 0.0 && arrival_radius < radius,
            "home area arrival radius must be positive and inside the wander radius"
        );
        Self {
            origin,
            radius,
            arrival_radius,
            return_blocked_timer: BlockedTimer::default(),
        }
    }

    pub(in crate::character) fn radius(&self) -> f32 {
        self.radius
    }

    pub(in crate::character) fn distance_from(&self, position: Vec2) -> f32 {
        self.origin.distance(position)
    }

    /// Whether `position` is within the arrival radius.
    pub(in crate::character) fn has_arrived(&self, position: Vec2) -> bool {
        self.distance_from(position) <= self.arrival_radius
    }

    /// The unit direction from `position` to the origin; `None` at the origin itself.
    pub(in crate::character) fn direction_home(&self, position: Vec2) -> Option<Vec2> {
        (self.origin - position).try_normalize()
    }

    /// Distance beyond the configured radius; negative while inside it.
    pub(in crate::character) fn overshoot(&self, position: Vec2) -> f32 {
        self.distance_from(position) - self.radius
    }

    /// [`Self::overshoot`] as a fraction of the radius; `None` unless the radius is finite and
    /// positive.
    pub(in crate::character) fn overshoot_fraction(&self, position: Vec2) -> Option<f32> {
        let radius = self.radius();
        if radius.is_finite() && radius > 0.0 {
            Some(self.overshoot(position) / radius)
        } else {
            None
        }
    }
}

/// Moves the home of each character whose return home is blocked to where it stands.
pub(super) fn move_homes_of_blocked_characters(
    time: Res<Time<Virtual>>,
    running_motions: Query<&RunningMotion>,
    mut characters: Query<(
        Option<&CurrentMotion>,
        &Transform,
        BlockedState,
        &mut HomeArea,
    )>,
) {
    for (current_motion, transform, blocked, mut home_area) in &mut characters {
        let motion = current_motion
            .and_then(|current_motion| running_motions.get(current_motion.get()).ok())
            .map(|running| running.motion);
        if motion != Some(Motion::ReturnToHome) {
            home_area.return_blocked_timer.reset();
            continue;
        }

        if home_area.return_blocked_timer.tick_and_check_limit(
            blocked.is_blocked(),
            time.delta_secs(),
            RETURN_BLOCKED_SECONDS,
        ) {
            home_area.origin = transform.translation.xy();
        }
    }
}
