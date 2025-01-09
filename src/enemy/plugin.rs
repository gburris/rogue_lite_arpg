use std::time::Duration;

use bevy::prelude::*;

use crate::{
    enemy::{
        events::DamageEvent,
        systems::{despawn_all_enemies, move_enemies_toward_player, spawn_enemies_with_timer},
    },
    labels::sets::GamePlaySet,
    labels::states::PlayingState,
    resources::EnemySpawnConfig,
};

use super::despawn_on_zero_health;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnConfig {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            quantity: 1,
        });
        app.add_systems(
            Update,
            (
                spawn_enemies_with_timer,
                move_enemies_toward_player,
                despawn_on_zero_health,
            )
                .in_set(GamePlaySet::Simulation)
                .run_if(in_state(PlayingState::Run)),
        )
        .add_observer(despawn_all_enemies)
        .add_event::<DamageEvent>();
    }
}
