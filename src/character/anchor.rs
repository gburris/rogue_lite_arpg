use bevy::prelude::*;

/// Marks a character's home position and how far it may wander from it.
#[derive(Component)]
pub struct Anchor {
    origin: Vec2,
    radius: f32,
}

impl Anchor {
    /// Creates an anchor at `origin` with the given wander radius in world units.
    pub fn new(origin: Vec2, radius: f32) -> Self {
        debug_assert!(
            radius.is_finite() && radius > 0.0,
            "anchor radius must be finite and positive"
        );
        Self { origin, radius }
    }

    /// Wander radius in world units.
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Distance from the anchor to `position`.
    pub fn distance_from(&self, position: Vec2) -> f32 {
        self.origin.distance(position)
    }

    /// Distance beyond the configured radius; negative while inside it.
    pub fn overshoot(&self, position: Vec2) -> f32 {
        self.distance_from(position) - self.radius
    }

    /// Overshoot as a fraction of the anchor range (`1.0` is one full range
    /// beyond it). Returns `None` for a non-positive or non-finite radius
    /// so callers treat it as having no anchor.
    pub fn overshoot_fraction(&self, position: Vec2) -> Option<f32> {
        let radius = self.radius();
        if radius.is_finite() && radius > 0.0 {
            Some(self.overshoot(position) / radius)
        } else {
            None
        }
    }

    /// The anchor's world-space origin.
    pub fn origin(&self) -> Vec2 {
        self.origin
    }
}
