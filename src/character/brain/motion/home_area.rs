use bevy::prelude::*;

/// A character's home: its center, how far it may wander from it, and how
/// close to the center counts as being back home.
#[derive(Component, Clone)]
pub struct HomeArea {
    origin: Vec2,
    radius: f32,
    arrival_radius: f32,
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
