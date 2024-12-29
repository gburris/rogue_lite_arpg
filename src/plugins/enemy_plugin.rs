use crate::systems::{move_enemies, spawn_enemies};
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Enemy Plugin added!");
        app.add_systems(Startup, spawn_enemies);
        app.add_systems(Update, move_enemies);
    }
}
