use bevy::prelude::*;

use crate::{enemy::systems::*, labels::sets::InGameSet};

use super::systems::enemy_movement::update_enemy_aim_position;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_enemies).add_systems(
            Update,
            (move_enemies_toward_player, update_enemy_aim_position).in_set(InGameSet::Simulation),
        );
    }
}
