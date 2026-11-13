use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use super::{BlockedTimer, Motion};
use crate::prelude::*;

/// Seconds a walk home pushes against a wall or a crowd before the character settles where it
/// stands.
const RETURN_BLOCKED_SECONDS: f32 = 1.0;

/// A character's home: its center, how far it may wander from it, and how
/// close to the center counts as being back home. A character that cannot walk back makes
/// the place it got stuck its new home.
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

    pub(in crate::character::brain) fn radius(&self) -> f32 {
        self.radius
    }

    pub(in crate::character::brain) fn distance_from(&self, position: Vec2) -> f32 {
        self.origin.distance(position)
    }

    /// Whether `position` is close enough to the origin to count as back home.
    pub(in crate::character::brain) fn has_arrived(&self, position: Vec2) -> bool {
        self.distance_from(position) <= self.arrival_radius
    }

    /// The unit direction from `position` to the origin; `None` at the origin itself.
    pub(in crate::character::brain) fn direction_home(&self, position: Vec2) -> Option<Vec2> {
        (self.origin - position).try_normalize()
    }

    /// Distance beyond the configured radius; negative while inside it.
    pub(in crate::character::brain) fn overshoot(&self, position: Vec2) -> f32 {
        self.distance_from(position) - self.radius
    }

    /// Overshoot as a fraction of the home area range (`1.0` is one full range
    /// beyond it). Returns `None` for a non-positive or non-finite radius
    /// so callers treat it as having no home area.
    pub(in crate::character::brain) fn overshoot_fraction(&self, position: Vec2) -> Option<f32> {
        let radius = self.radius();
        if radius.is_finite() && radius > 0.0 {
            Some(self.overshoot(position) / radius)
        } else {
            None
        }
    }
}

/// Moves the home of each character whose walk back is blocked to where it stands.
pub(super) fn settle_blocked_returners(
    time: Res<Time<Virtual>>,
    mut returners: Query<(
        &Brain,
        &Transform,
        &Kinematics,
        &StatMultipliers,
        &LinearVelocity,
        &mut HomeArea,
    )>,
) {
    for (brain, transform, kinematics, stat_multipliers, linear_velocity, mut home_area) in
        &mut returners
    {
        if brain.motion() != Some(Motion::ReturnToHome) {
            home_area.return_blocked_timer.reset();
            continue;
        }
        let is_blocked = kinematics.is_blocked_at(linear_velocity.0, stat_multipliers.speed);
        if home_area.return_blocked_timer.tick_and_check_limit(
            is_blocked,
            time.delta_secs(),
            RETURN_BLOCKED_SECONDS,
        ) {
            home_area.origin = transform.translation.xy();
        }
    }
}
