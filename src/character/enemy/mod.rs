use bevy::prelude::*;

mod bat;
mod defeat;
mod kind;
mod mage;
mod spawn;
mod warrior;

pub use self::defeat::Experience;
pub use self::kind::EnemyKind;
pub use self::spawn::{EnemySpawn, SpawnEnemies};
use self::spawn::{
    enemy_children, enemy_wander, ground_sight_ray, sight_ray, spawn_enemy_with_equipment,
};
use crate::character::Character;
use crate::character::brain::VisionCone;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn::on_spawn_enemies);
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
