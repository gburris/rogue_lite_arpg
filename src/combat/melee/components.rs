use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{
        damage::components::CollisionDamage, status_effects::components::EffectsList,
        weapon::weapon::Weapon,
    },
    configuration::GameCollisionLayer,
};

#[derive(Component, Clone)]
#[require(Weapon)]
pub struct MeleeWeapon {
    pub melee_attack: MeleeSwingPropertiesBundle,
}

#[derive(Component, Clone)]

pub struct MeleeHitbox {
    pub length: f32,
    pub width: f32,
}

impl Default for MeleeHitbox {
    fn default() -> Self {
        MeleeHitbox {
            length: 40.0,
            width: 10.0,
        }
    }
}
#[derive(Component, Debug, Clone)]
pub enum MeleeSwingType {
    Stab { speed: f32, duration: f32 },
    Slash { radius: f32, duration: f32 },
}

impl Default for MeleeSwingType {
    fn default() -> Self {
        MeleeSwingType::stab()
    }
}

impl MeleeSwingType {
    pub fn stab() -> Self {
        MeleeSwingType::Stab {
            speed: 10.0,
            duration: 0.4,
        }
    }

    pub fn slash() -> Self {
        MeleeSwingType::Slash {
            radius: 25.0,
            duration: 0.4, // Complete in 0.4s
        }
    }

    pub fn get_total_duration(&self) -> f32 {
        match self {
            MeleeSwingType::Stab { duration, .. } => *duration,
            MeleeSwingType::Slash { duration, .. } => *duration,
        }
    }
}
#[derive(Bundle, Clone)]
pub struct MeleeSwingPropertiesBundle {
    pub damage: CollisionDamage,
    pub hitbox: MeleeHitbox,
    pub swing_type: MeleeSwingType,
    pub effects_list: EffectsList,
}

#[derive(Component)]
#[require(
    CollidingEntities,
    CollisionDamage,
    Sensor,
    CollisionLayers(default_collision_layers)
)]
pub struct ActiveMeleeAttack {
    pub timer: Timer,
    pub initial_angle: f32,
    pub attack_type: MeleeSwingType,
}

fn default_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameCollisionLayer::Grounded, [GameCollisionLayer::Enemy])
}
