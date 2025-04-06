use bevy::prelude::*;

/** Represents an entity that will be despawned after time elapsed */
#[derive(Component)]
pub struct LiveDuration(pub Timer);

impl LiveDuration {
    pub fn new(duration_secs: f32) -> Self {
        LiveDuration(Timer::from_seconds(duration_secs, TimerMode::Once))
    }
}

impl Default for LiveDuration {
    fn default() -> Self {
        LiveDuration(Timer::from_seconds(2.0, TimerMode::Once))
    }
}
