use bevy::prelude::*;

use crate::{
    combat::{
        damage::components::CollisionDamage, spells::components::Spell,
        status_effects::components::EffectsList,
    },
    despawn::components::LiveDuration,
};

#[derive(Component, Clone, Default)]
#[require(LiveDuration)]
pub struct Projectile;

#[derive(Bundle, Clone)]
pub struct ProjectileBundle {
    pub sprite: Sprite,
    pub damage: CollisionDamage,
    pub effects_list: EffectsList,
    pub spell: Spell, // maybe just make this a part of projectile
}
