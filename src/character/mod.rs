mod animation;
mod behavior;
mod enemy;
mod npc;
mod player;
mod state;
mod vision;

pub mod prelude {
    pub use super::enemy::*;
    pub use super::npc::*;
    pub use super::player::prelude::*;
    pub use super::state::*;
    pub use super::vision::*;
}

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{character::animation::CharacterAnimationState, prelude::*};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((player::plugin, enemy::plugin, npc::plugin));

        app.add_plugins((animation::plugin, behavior::plugin, vision::plugin));

        app.add_systems(
            FixedUpdate,
            state::motion_to_velocity.in_set(MainSystems::InGame),
        );
    }
}

#[derive(Component, Default)]
#[require(RigidBody,
    LockedAxes = LockedAxes::new().lock_rotation(),
    TransformInterpolation,
    // Set stable mass for characters so speed can be compared numerically
    Mass(50.0),
    NoAutoMass,
    CharacterAnimationState,
    Vision,
    ItemCapacity(10),
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

#[derive(Component, Clone, Default)]
pub struct Purse {
    pub amount: u32,
}

impl Purse {
    pub fn add(&mut self, amount: u32) {
        self.amount += amount;
    }

    // pub fn remove(&mut self, amount: u32) -> Result<u32, String> {
    //     if self.amount >= amount {
    //         self.amount -= amount;
    //         Ok(self.amount)
    //     } else {
    //         Err("Not enough in purse!".to_string())
    //     }
    // }
}
