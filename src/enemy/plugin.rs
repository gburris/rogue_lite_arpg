use bevy::prelude::*;

use crate::{enemy::systems::*, labels::sets::InGameSet};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_enemy_assets)
            .add_observer(spawn_enemies)
            .add_systems(
                Update,
                (move_enemies_toward_player).in_set(InGameSet::Simulation),
            );
    }
}
