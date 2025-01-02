use bevy::{
    math::Vec3,
    prelude::{Commands, Entity},
};

use crate::events::EnemyDefeatedEvent;

pub fn handle_enemy_death(commands: &mut Commands, entity: Entity, position: Vec3, exp_value: u32) {
    //TODO Make a helper for all enemies damage / dying logic
    return;
}
