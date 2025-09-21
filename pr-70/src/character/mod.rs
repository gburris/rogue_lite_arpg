mod behavior;
pub mod enemy;
pub mod npc;
pub mod player;
mod simple_motion;
mod state;
mod vision;

pub mod prelude {
    pub use crate::character::enemy::Enemy;
    pub use crate::character::npc::NPC;
    pub use crate::character::player::interact::PlayerInteractionRadius;
    pub use crate::character::player::Player;
    pub use crate::character::simple_motion::SimpleMotion;
    pub use crate::character::state::*;
    pub use crate::character::vision::Vision;
}

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::AnimationTimer,
    configuration::{GameCollisionLayer, YSort, CHARACTER_FEET_POS_OFFSET},
    labels::sets::{InGameSet, MainSet},
};

use enemy::EnemyPlugin;
use npc::NPCPlugin;
use player::PlayerPlugin;
use state::ActionState;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerPlugin, EnemyPlugin, NPCPlugin));

        app.add_systems(
            FixedUpdate,
            simple_motion::to_velocity.in_set(MainSet::InGame),
        );

        app.add_systems(
            Update,
            (
                state::update_state_on_simple_motion_change,
                behavior::while_chasing,
                behavior::while_idling,
                behavior::while_wandering,
                behavior::while_retreating,
                behavior::while_keeping_distance_and_firing,
                // Vision + Perception
                vision::update_aim_position,
                vision::update_target_info,
                vision::is_target_in_sight,
                vision::debug_vision,
                // Targeting
                vision::should_target_watched,
                vision::should_stop_targeting,
            )
                .in_set(InGameSet::Simulation),
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
