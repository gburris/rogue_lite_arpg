use bevy::prelude::*;

use crate::combat::projectile::components::ProjectileBundle;

#[derive(Component, Default)]
pub struct Weapon;

#[derive(Component)]
#[require(Weapon)]
pub struct ProjectileWeapon {
    pub projectile: ProjectileBundle,
    pub projectile_speed: f32,
    pub spread: f32,
}
