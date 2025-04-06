use bevy::prelude::Resource;
use bevy::prelude::Timer;

#[derive(Resource)]
pub struct EnemySpawnConfig {
    /// How often to spawn a new enemy? (repeating timer)
    pub timer: Timer,
    pub quantity: u32,
}
