//Shared component between all projectile conpoments
//This marks something as a projectile when it's a component of it
//Projectile systems will move it and detect collision with the player and enimes
use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
}