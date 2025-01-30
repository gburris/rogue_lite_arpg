use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{
        damage::components::CollisionDamage, status_effects::components::EffectsList,
        weapon::weapon::Weapon,
    },
    configuration::GameCollisionLayer,
};

//Big TODO: Move basically everything in this into this component so there is just one component to
//Repesent a melee weapon
#[derive(Component, Clone)]
#[require(Weapon)]
pub struct MeleeWeapon {
    pub swing_duration: f32,
    pub damage: CollisionDamage,
    pub hitbox: MeleeHitbox,
    pub swing_type: MeleeSwingType,
    pub effects_list: EffectsList,
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
