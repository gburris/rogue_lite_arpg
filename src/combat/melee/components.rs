use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::{damage::components::DamageSource, status_effects::components::EffectsList},
    configuration::GameCollisionLayer,
};

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

#[derive(Component)]
#[require(CollidingEntities, Sensor)]
pub struct ActiveMeleeAttack {
    pub initial_angle: f32,
    pub entities_damaged: Vec<Entity>,
}

pub fn calculate_damage(damage_range: (f32, f32)) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(damage_range.0..damage_range.1)
}
