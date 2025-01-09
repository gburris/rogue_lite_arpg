//Shared component between all projectile conpoments
//This marks something as a projectile when it's a component of it
//Projectile systems will move it and detect collision with the player and enimes
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Projectile {
    pub motion: f32,
    pub time_to_live: Timer,
}

impl Projectile {
    pub fn new(motion: f32) -> Self {
        Projectile {
            motion,
            time_to_live: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}
