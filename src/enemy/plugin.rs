use std::time::Duration;

use bevy::prelude::*;

use crate::{enemy::systems::*, labels::sets::InGameSet, labels::states::InGameState};

use super::resources::EnemySpawnConfig;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnConfig {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            quantity: 1,
        })
        .add_systems(Startup, setup_enemy_assets)
        .add_systems(
            Update,
            (spawn_enemies_with_timer, move_enemies_toward_player)
                .in_set(InGameSet::Simulation)
                .run_if(in_state(InGameState::Run)),
        );
    }
}
