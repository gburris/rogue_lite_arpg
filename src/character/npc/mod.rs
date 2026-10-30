use bevy::prelude::*;

mod interaction;
mod kind;
mod spawn;
mod villager;

pub use self::kind::NpcKind;
pub use self::spawn::{NpcSpawnData, SpawnNpcs};
pub(in crate::character) use self::villager::VILLAGER;
use crate::character::Character;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn::spawn_npcs);
}

#[derive(Component)]
#[require(Character, CombatSide = CombatSide::Ally, DespawnOnExit::<AppState>(AppState::Playing))]
pub struct Npc;
