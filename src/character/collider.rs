use avian2d::prelude::*;
use bevy::prelude::*;

use crate::prelude::*;

/// Circle collider at the character's feet that collides with the ground and obstacles.
pub fn physical_collider() -> impl Bundle {
    (
        Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
        Collider::circle(10.0),
        CollisionLayers::new(
            [GameCollisionLayer::Grounded],
            [
                GameCollisionLayer::Grounded,
                GameCollisionLayer::HighObstacle,
                GameCollisionLayer::LowObstacle,
            ],
        ),
    )
}
