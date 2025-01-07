use bevy::prelude::*;

/** Represents an entity that will be despawned after time elapsed */
#[derive(Component)]
pub struct Duration(pub Timer);

impl Duration {
    pub fn new(duration: f32) -> Self {
        Duration(Timer::from_seconds(duration, TimerMode::Once))
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration(Timer::from_seconds(2.0, TimerMode::Once))
    }
}
