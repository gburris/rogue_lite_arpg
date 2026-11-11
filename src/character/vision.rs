use bevy::prelude::*;

/// Represents the current direction an entity is aiming toward (e.g., cursor for player, target for
/// AI). This is decoupled from movement-facing direction.
#[derive(Component, Default)]
pub struct Vision {
    pub aim_direction: Vec2,
}

/// Stores calculated perception data for a given potential or current target.
#[derive(Component, Default, Clone)]
pub struct TargetInfo {
    /// Distance to the observed entity.
    pub distance: f32,
    /// Direction vector pointing to the observed entity.
    pub direction: Vec2,
    /// World position of the observed entity.
    pub position: Vec2,
    /// Intended world velocity of the observed entity.
    pub velocity: Vec2,
    /// Whether the observed entity is within an unobstructed line of sight (based on `RayCaster`).
    pub line_of_sight: bool,
    /// Whether the observed entity is within the entity’s vision cone angle.
    pub in_vision_cone: bool,
    /// When the current `Targeting` was acquired, in virtual seconds.
    pub targeted_at: f32,
    // TODO: Add `last_known_position` for memory/prediction systems.
}

impl TargetInfo {
    /// Records where the observed entity is relative to `observer_position`, and how it moves.
    pub fn record_observation(
        &mut self,
        observer_position: Vec2,
        target_position: Vec2,
        target_velocity: Vec2,
    ) {
        self.direction = (target_position - observer_position).normalize_or_zero();
        self.distance = target_position.distance(observer_position);
        self.position = target_position;
        self.velocity = target_velocity;
    }

    pub fn has_finite_distance(&self) -> bool {
        self.distance.is_finite()
    }

    /// Whether the observed entity is at a finite distance of at most `reach`.
    pub fn is_within(&self, reach: f32) -> bool {
        self.has_finite_distance() && self.distance <= reach
    }

    /// The direction to the observed entity; `None` while the two overlap, leaving no
    /// direction to aim along.
    pub fn nonzero_direction(&self) -> Option<Vec2> {
        (self.direction.length_squared() > f32::EPSILON).then_some(self.direction)
    }

    /// Whether the observed entity is in line of sight and inside the vision cone.
    pub fn is_in_view(&self) -> bool {
        self.line_of_sight && self.in_vision_cone
    }
}
