use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    ai::state::ActionState,
    animation::AnimationTimer,
    configuration::{GameCollisionLayer, YSort, CHARACTER_FEET_POS_OFFSET},
};

#[derive(Component, Default)]
#[require(RigidBody,
    LockedAxes = LockedAxes::new().lock_rotation(),
    TransformInterpolation,
    // Set stable mass for characters so speed can be compared numerically
    Mass(50.0),
    NoAutoMass,
    ActionState,
    AnimationTimer,
    YSort::from_offset(CHARACTER_FEET_POS_OFFSET))]
pub struct Character;

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
