mod ability;
mod animation;
mod brain;
mod enemy;
mod motion;
mod npc;
mod player;
mod state;

use avian2d::prelude::*;
use bevy::prelude::*;

use self::animation::{AnimationSheet, CharacterAnimationState};
pub(in crate::character) use self::state::{Allies, physical_collider};
use crate::prelude::*;

pub mod prelude {
    pub use super::Character;
    pub use super::ability::{Abilities, Ability};
    #[cfg(feature = "dev")]
    pub(crate) use super::brain::TargetInfo;
    #[cfg(feature = "dev_native")]
    pub(crate) use super::brain::lock_onto_target;
    pub use super::brain::{Brain, ScoreTrace};
    #[cfg(feature = "dev_tools")]
    pub(crate) use super::brain::{TargetLock, Targeting, VisionCone};
    pub use super::enemy::{Enemy, EnemyKind, EnemySpawn, SpawnEnemies};
    pub use super::motion::{MotionKey, MotionSources, RunningMotion, Wander, WanderProgress};
    pub use super::npc::{Npc, NpcKind, NpcSpawn, SpawnNpcs};
    pub use super::player::prelude::{
        Attribute, GameProgress, InteractionZone, Player, PlayerAim, PlayerAttributes,
        PlayerInteraction, PlayerInteractionRadius,
    };
    #[cfg(feature = "dev_native")]
    pub(crate) use super::player::prelude::{
        PlayerMovement, UseMainhand, UseOffhand, place_player_aim,
    };
    pub use super::state::{
        FacingDirection, InterruptionState, Kinematics, KnockedBack, Purse, Vision,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, enemy::plugin, npc::plugin));

    app.add_plugins((animation::plugin, brain::plugin, motion::plugin));

    app.add_systems(
        FixedUpdate,
        state::kinematics_to_velocity.in_set(MainSystems::InGame),
    );
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
