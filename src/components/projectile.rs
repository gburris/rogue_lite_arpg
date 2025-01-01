//Shared component between all projectile conpoments
//This marks something as a projectile when it's a component of it
//Projectile systems will move it and detect collision with the player and enimes
use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
    pub time_to_live: Timer,
}

impl Projectile {
    pub fn new(speed: f32) -> Self {
        Projectile {
            speed,
            time_to_live: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}
