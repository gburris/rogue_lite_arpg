use avian2d::prelude::*;
use bevy::prelude::*;
use enemy::EnemyPlugin;
use npc::NPCPlugin;
use player::PlayerPlugin;

use crate::{
    ai::state::ActionState,
    animation::AnimationTimer,
    configuration::{GameCollisionLayer, YSort, CHARACTER_FEET_POS_OFFSET},
};

pub mod enemy;
pub mod npc;
pub mod player;

pub mod prelude {
    pub use crate::character::enemy::Enemy;
    pub use crate::character::npc::NPC;
    pub use crate::character::player::interact::PlayerInteractionRadius;
    pub use crate::character::player::Player;
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerPlugin, EnemyPlugin, NPCPlugin));
    }
}

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
