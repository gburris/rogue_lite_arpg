mod anchor;
mod animation;
mod brain;
mod enemy;
mod npc;
mod player;
mod state;
mod vision;

pub mod prelude {
    pub use super::brain::BrainKind;
    pub use super::enemy::*;
    pub use super::npc::*;
    pub use super::player::prelude::*;
    pub use super::state::*;
    pub use super::vision::*;
}

/// Ordered AI phases. Perception feeds targeting, aim, decisions, execution
/// of those decisions, and finally the motion driver.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AiSet {
    Perception,
    Targeting,
    Aim,
    Decision,
    Execute,
    Motion,
}

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    character::animation::{AnimationSheet, CharacterAnimationState},
    prelude::*,
};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((player::plugin, enemy::plugin, npc::plugin));

        app.add_plugins((animation::plugin, vision::plugin, brain::plugin));

        app.configure_sets(
            Update,
            (
                AiSet::Perception,
                AiSet::Targeting,
                AiSet::Aim,
                AiSet::Decision,
                AiSet::Execute,
                AiSet::Motion,
            )
                .chain()
                .in_set(SimulationSystems::Logic),
        );

        app.add_systems(
            FixedUpdate,
            state::motion_to_velocity.in_set(MainSystems::InGame),
        );

        app.add_systems(
            Update,
            state::sync_attack_state_from_equipment.in_set(SimulationSystems::Derive),
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
    AnimationSheet,
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
