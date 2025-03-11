use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{damage::DamageSource, status_effects::components::EffectsList},
    configuration::GameCollisionLayer,
};

mod melee_attack;

pub use melee_attack::*;

//Repesent a melee weapon
#[derive(Component, Clone)]
pub struct MeleeWeapon {
    pub attack_duration: Timer,
    pub damage: (f32, f32),
    pub hitbox: Collider,
    pub effects_list: EffectsList,
    pub attack_type: MeleeSwingType,
}

impl MeleeWeapon {
    /// Gets collision layers for melee weapon based on source of damage
    ///
    /// This is meant to be added when the weapon is equipped.
    /// We consider melee weapons "Grounded" so they can be used to break chests, etc... on the ground
    pub fn collision_layers(damage_source: DamageSource) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::Grounded, LayerMask::from(damage_source))
    }
}

#[derive(Debug, Clone)]
pub enum MeleeSwingType {
    Stab { speed: f32 },
    Slash { radius: f32 },
}

impl MeleeSwingType {
    pub fn stab() -> Self {
        MeleeSwingType::Stab { speed: 10.0 }
    }

    pub fn slash() -> Self {
        MeleeSwingType::Slash { radius: 25.0 }
    }
}
