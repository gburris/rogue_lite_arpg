use bevy::prelude::*;

use crate::enemy::CollisionDamage;

#[derive(Event)]
pub struct EnemyCollidesWithPlayer {
    pub collision_damage: CollisionDamage,
}
