use bevy::prelude::*;

use crate::{combat::projectile::components::ProjectileBundle, items::Equippable};

#[derive(Component, Default)]
pub struct Weapon;

#[derive(Component)]
#[require(Weapon)]
pub struct ProjectileWeapon {
    pub projectile: ProjectileBundle,
    pub spread: f32,
}

pub fn tick_equippable_use_rate(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in equippable_query.iter_mut() {
        equippable.use_rate.tick(time.delta());
    }
}
