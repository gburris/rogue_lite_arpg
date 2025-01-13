//Shared component between all projectile conpoments
//This marks something as a projectile when it's a component of it
//Projectile systems will move it and detect collision with the player and enimes
use bevy::prelude::*;

use crate::despawn::components::LiveDuration;

#[derive(Component, Default)]
#[require(LiveDuration)]
pub struct Projectile {
    pub speed: f32,
}

impl Projectile {
    pub fn new(speed: f32) -> Self {
        Projectile { speed }
    }
}
