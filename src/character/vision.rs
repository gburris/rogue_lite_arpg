use bevy::prelude::*;

/// Represents the current direction an entity is aiming toward (e.g., cursor for player, target for
/// AI). This is decoupled from movement-facing direction.
#[derive(Component, Default)]
pub struct Vision {
    pub aim_direction: Vec2,
}

/// Stores calculated perception data for a given potential or current target.
#[derive(Component, Default)]
pub struct TargetInfo {
    /// Distance to the observed entity.
    pub distance: f32,
    /// Direction vector pointing to the observed entity.
    pub direction: Vec2,
    /// World position of the observed entity.
    pub target_position: Vec2,
    /// Intended world velocity of the observed entity.
    pub target_velocity: Vec2,
    /// Whether the observed entity is within an unobstructed line of sight (based on `RayCaster`).
    pub line_of_sight: bool,
    /// Whether the observed entity is within the entity’s vision cone angle.
    pub in_vision_cone: bool,
    /// When the current `Targeting` was acquired, in virtual seconds.
    pub targeted_at: f32,
    // TODO: Add `last_known_position` for memory/prediction systems.
}
