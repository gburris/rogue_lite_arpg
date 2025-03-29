use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{damage::DamageSource, status_effects::components::EffectsList},
    configuration::GameCollisionLayer,
};

mod melee_attack;

pub use melee_attack::*;

/// Our pixel weapons all face upwards currently, so we must rotate them 90 degrees for attacks to
/// occur in the direction we expect. This value will need to be updated if our assets change
pub const MELEE_WEAPON_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

//Repesent a melee weapon
#[derive(Component, Clone)]
pub struct MeleeWeapon {
    // Time it takes (seconds) to complete the attack, smaller = faster
    pub attack_time: f32,
    pub damage: (f32, f32),
    pub hitbox: Collider,
    pub effects_list: EffectsList,
    pub attack_type: MeleeSwingType,
    pub hold_distance: f32,
}

impl MeleeWeapon {
    /// Gets collision layers for melee weapon based on source of damage
    /// It can either target allies or enemies
    pub fn collision_layers(damage_source: DamageSource) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HitBox, LayerMask::from(damage_source))
    }
}

#[derive(Debug, Clone)]
pub enum MeleeSwingType {
    Stab {
        /// How far the weapon should move (stab) forward from its starting position
        reach: f32,
    },
    Slash {
        /// Distance we want slash to travel in radians
        arc_distance: f32,
    },
}

impl MeleeSwingType {
    pub const STAB: Self = MeleeSwingType::Stab { reach: 30.0 };
    pub const SLASH: Self = MeleeSwingType::Slash {
        arc_distance: 180f32.to_radians(),
    };
}
