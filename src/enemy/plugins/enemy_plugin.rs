use crate::{
    enemy::{
        handle_enemy_damage, systems::{move_enemies_toward_player, spawn_enemies_with_timer}, EnemyDamageEvent
    },
    labels::sets::GamePlaySet,
    resources::EnemySpawnConfig,
};
use std::time::Duration;

use bevy::prelude::*;

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
                handle_enemy_damage,
            )
                .in_set(GamePlaySet::Simulation),
        )
        .add_event::<EnemyDamageEvent>();
    }
}
