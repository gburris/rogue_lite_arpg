use bevy::prelude::*;

mod bat;
mod mage;
mod spawn;
mod warrior;

pub use self::spawn::{EnemyKind, EnemySpawn, SpawnEnemies};
use self::spawn::{
    Experience, enemy_children, enemy_motion_sources, ground_sight_ray, sight_ray,
    spawn_enemy_with_equipment,
};
use crate::character::Character;
use crate::character::brain::VisionCone;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn::on_spawn_enemies);
}

#[derive(Component, Default, Clone)]
#[require(
    Character,
    VisionCone,
    Purse { amount: 50 },
    CombatSide = CombatSide::Enemy,
    DespawnOnExit::<AppState>(AppState::Playing)
)]
pub struct Enemy;
