mod animation;
mod behavior;
pub mod enemy;
pub mod npc;
pub mod player;
mod state;
mod vision;

pub mod prelude {
    pub use super::animation::*;
    pub use super::enemy::*;
    pub use super::npc::*;
    pub use super::player::interact::PlayerInteractionRadius;
    pub use super::player::*;
    pub use super::state::*;
    pub use super::vision::Vision;
}

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{items::ItemCapacity, prelude::*};

use player::PlayerPlugin;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerPlugin, enemy::plugin, npc::plugin, animation::plugin));

        app.add_systems(
            FixedUpdate,
            state::motion_to_velocity.in_set(MainSystems::InGame),
        );

        app.add_systems(
            Update,
            (
                behavior::while_chasing,
                behavior::while_idling,
                behavior::while_wandering,
                behavior::while_retreating,
                behavior::while_keeping_distance_and_firing,
                // Vision + Perception
                vision::update_aim_position,
                vision::update_target_info,
                vision::is_target_in_sight,
                // Targeting
                vision::should_target_watched,
                vision::should_stop_targeting,
            )
                .in_set(InGameSystems::Simulation),
        )
        .add_observer(behavior::on_idle_start)
        .add_observer(behavior::on_wander_start)
        .add_observer(behavior::on_attempt_melee)
        .add_observer(vision::on_damage_aggro);
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

    pub fn remove(&mut self, amount: u32) -> Result<u32, String> {
        if self.amount >= amount {
            self.amount -= amount;
            Ok(self.amount)
        } else {
            Err("Not enough in purse!".to_string())
        }
    }
}
