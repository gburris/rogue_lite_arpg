use bevy::prelude::*;

use crate::{enemy::systems::*, items::RechargeEvent, labels::sets::InGameSet};

use super::systems::enemy_movement::update_enemy_aim_position;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_enemy_assets)
            .add_observer(spawn_enemies)
            .add_event::<RechargeEvent>()
            .add_observer(on_enemy_defeated)
            .add_systems(
                Update,
                (move_enemies_toward_player, update_enemy_aim_position)
                    .in_set(InGameSet::Simulation),
            );
    }
}
