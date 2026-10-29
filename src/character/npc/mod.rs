use bevy::prelude::*;

mod interaction;
mod spawn;

use super::brain::considerations::{ACTION_BASELINE_IDLE, RETURN_TO_HOME, WANDER};
use super::brain::{Behavior, BrainDefinition};
use crate::character::Character;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn::spawn_npcs);
}

#[derive(Event)]
pub struct SpawnNpcs(pub Vec<NpcSpawnData>);

#[derive(Debug, Clone)]
pub struct NpcSpawnData {
    pub position: Vec2,
    pub npc_kind: NpcKind,
}

#[derive(Component)]
#[require(Character, CombatSide = CombatSide::Ally, DespawnOnExit::<AppState>(AppState::Playing))]
pub struct NPC;

/// Recipes name these directly, e.g. `(what: Npc(Shopkeeper), ...)` in `map_recipes.ron`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Deserialize)]
pub enum NpcKind {
    Helper,
    Shopkeeper,
    StatTrainer,
}

static VILLAGER_BEHAVIORS: &[Behavior] = &[WANDER, ACTION_BASELINE_IDLE, RETURN_TO_HOME];

/// Wanders around its home area and never requests equipment.
pub(in crate::character) static VILLAGER: BrainDefinition = BrainDefinition {
    behaviors: VILLAGER_BEHAVIORS,
};

impl NpcKind {
    pub fn brain(&self) -> &'static BrainDefinition {
        &VILLAGER
    }
}
