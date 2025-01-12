use std::time::Duration;

use bevy::prelude::*;

use crate::{
    enemy::systems::*, labels::sets::GamePlaySet, labels::states::PlayingState,
    resources::EnemySpawnConfig,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnConfig {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            quantity: 1,
        })
        .add_systems(
            Update,
            (spawn_enemies_with_timer, move_enemies_toward_player)
                .in_set(GamePlaySet::Simulation)
                .run_if(in_state(PlayingState::Run)),
        );
    }
}
