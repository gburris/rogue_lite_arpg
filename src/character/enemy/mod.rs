use bevy::prelude::*;

mod bat;
mod defeat;
mod mage;
mod spawn;
mod warrior;

use self::spawn::{
    enemy_children, enemy_wander, ground_sight_ray, sight_ray, spawn_enemy_with_equipment,
};
use crate::character::brain::{BrainDefinition, VisionCone};
use crate::character::{Character, Purse};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn::spawn_enemies);
}

#[derive(Debug, Event)]
pub struct SpawnEnemies(pub Vec<EnemySpawnData>);

#[derive(Debug, Clone, Copy)]
pub struct EnemySpawnData {
    pub position: Vec2,
    pub enemy_kind: EnemyKind,
}

#[derive(Component)]
#[require(
    Character,
    VisionCone,
    Purse { amount: 50 },
    CombatSide = CombatSide::Enemy,
    DespawnOnExit::<AppState>(AppState::Playing)
)]
pub struct Enemy;

/// Recipes name these directly, e.g. `(what: Enemy(FireMage), ...)` in `map_recipes.ron`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Deserialize)]
pub enum EnemyKind {
    Warrior,
    IceMage,
    FireMage,
    /// Flying enemy: circles its target and breathes fire without a weapon.
    Bat,
}

impl EnemyKind {
    /// Every enemy kind, used for random spawns and exhaustive iteration.
    pub const ALL: &[Self] = &[Self::Warrior, Self::IceMage, Self::FireMage, Self::Bat];

    pub fn brain(self) -> &'static BrainDefinition {
        match self {
            Self::Warrior => &warrior::WARRIOR,
            Self::IceMage => &mage::ICE_MAGE,
            Self::FireMage => &mage::FIRE_MAGE,
            Self::Bat => &bat::BAT,
        }
    }
}

/// Experience paid out when the player defeats this enemy. Scales with
/// threat: the tanky warrior pays most, the fragile flying bat least.
#[derive(Component)]
pub struct Experience {
    pub base_exp: f32,
}
