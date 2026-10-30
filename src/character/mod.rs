mod animation;
mod brain;
mod collider;
mod enemy;
mod npc;
mod player;
mod purse;
mod state;
mod vision;

use avian2d::prelude::*;
use bevy::prelude::*;

use self::animation::{AnimationSheet, CharacterAnimationState};
pub(in crate::character) use self::collider::physical_collider;
use crate::prelude::*;

pub mod prelude {
    pub use super::Character;
    pub(crate) use super::brain::Targeting;
    pub use super::brain::{AiSet, Brain, Dash, Motion, ScoreTrace, Wander};
    pub use super::enemy::{Enemy, EnemyKind, EnemySpawnData, Experience, SpawnEnemies};
    pub use super::npc::{Npc, NpcKind, NpcSpawnData, SpawnNpcs};
    pub use super::player::prelude::{
        DisplayableStatType, GameProgress, InteractionZone, Player, PlayerAim, PlayerInteraction,
        PlayerInteractionRadius, PlayerStats,
    };
    pub use super::purse::Purse;
    pub use super::state::{AttackState, FacingDirection, Kinematics};
    pub use super::vision::{TargetInfo, Vision};
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((player::plugin, enemy::plugin, npc::plugin));

        app.add_plugins((animation::plugin, brain::plugin));

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
            state::kinematics_to_velocity.in_set(MainSystems::InGame),
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
