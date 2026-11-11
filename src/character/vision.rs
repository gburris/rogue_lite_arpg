use bevy::prelude::*;

/// Where an entity aims: a direction and how far along it (the cursor for the player, the target
/// for AI). This is decoupled from movement-facing direction.
#[derive(Component)]
pub struct Vision {
    pub aim_direction: Vec2,
    pub aim_distance: f32,
}

impl Default for Vision {
    fn default() -> Self {
        Self {
            aim_direction: Vec2::ZERO,
            // A holder that never aims still fires a sane distance away from itself.
            aim_distance: 100.0,
        }
    }
}
