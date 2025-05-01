mod behavior;
pub mod enemy;
pub mod npc;
pub mod player;
mod simple_motion;
mod state;

pub mod prelude {
    pub use crate::character::enemy::Enemy;
    pub use crate::character::npc::NPC;
    pub use crate::character::player::interact::PlayerInteractionRadius;
    pub use crate::character::player::Player;
    pub use crate::character::simple_motion::SimpleMotion;
    pub use crate::character::state::*;
    pub use crate::character::Vision;
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
                behavior::check_for_target_interrupt,
                behavior::while_chasing,
                behavior::while_idling,
                behavior::while_wandering,
                behavior::while_retreating,
            )
                .in_set(InGameSet::Simulation),
        )
        .add_observer(behavior::on_idle_start)
        .add_observer(behavior::on_wander_start);
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

/// Represents the world coordinate where an entitiy is aiming, for player this is the cursor
#[derive(Component)]
pub struct Vision {
    pub aim_position: Vec2,
}

impl Default for Vision {
    fn default() -> Self {
        Self {
            aim_position: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct Agro {
    pub target: Option<Entity>,
    pub line_of_sight: bool,
    pub target_lock_timer: Option<Timer>,
    lock_duration: f32,
}

impl Default for Agro {
    fn default() -> Self {
        Self {
            target: None,
            line_of_sight: false,
            target_lock_timer: None,
            // enemies chase for 6 seconds when damaged by default
            lock_duration: 6.0,
        }
    }
}

impl Agro {
    pub fn lock_target(&mut self, target: Entity) {
        self.target_lock_timer = Some(Timer::from_seconds(self.lock_duration, TimerMode::Once));
        self.target = Some(target);
    }

    pub fn has_target(&self) -> bool {
        self.target.is_some()
    }
}

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
