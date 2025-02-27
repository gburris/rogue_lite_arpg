use bevy::prelude::*;

use crate::combat::projectile::components::ProjectileBundle;
#[derive(Component)]
pub struct ProjectileWeapon {
    pub projectile: ProjectileBundle,
    pub projectile_speed: f32,
    pub spread: f32,
}
