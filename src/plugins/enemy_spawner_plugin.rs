use crate::{
    resources::EnemySpawnConfig,
    systems::{move_enemies_toward_player, spawn_enemies_with_timer},
};
use std::time::Duration;

use bevy::prelude::*;

pub struct EnemySpawnerPlugin;

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Player Plugin! added!");
        app.insert_resource(EnemySpawnConfig {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            quantity: 1,
        });
        app.add_systems(Update, spawn_enemies_with_timer);
        app.add_systems(Update, move_enemies_toward_player);
    }
}
