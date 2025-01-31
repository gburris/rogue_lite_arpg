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
    pub attack_duration: Timer,
    pub damage: f32,
    pub hitbox: MeleeHitbox,
    pub effects_list: EffectsList,
    pub attack_type: MeleeSwingType,
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
    pub initial_angle: f32,
}

fn default_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameCollisionLayer::Grounded, [GameCollisionLayer::Enemy])
}
