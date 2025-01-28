use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{damage::components::CollisionDamage, status_effects::components::EffectsList, weapon::weapon::Weapon},
    configuration::GameCollisionLayer,
    despawn::components::LiveDuration,
};



#[derive(Component, Clone)]
#[require(Weapon)]
pub struct MeleeWeapon {
    pub melee_attack: MeleeSwingPropertiesBundle,
}


#[derive(Component, Clone)]

pub struct MeleeHitbox{
    pub length: f32, 
    pub width: f32, 
}

impl Default for MeleeHitbox {
    fn default() -> Self {
        MeleeHitbox {
            length: 500.0,
            width: 50.0,
        }
    }
}
#[derive(Component, Debug, Clone)]
pub enum MeleeSwingType {
    Stab {
        speed: f32,
        duration: f32,
    },
    Slash {
        angular_speed: f32,
        radius: f32,
        current_angle: f32,
        duration: f32,
        elapsed_time: f32,
    },
    Circle {
        expansion_speed: f32,
        current_radius: f32,
        duration: f32,
        elapsed_time: f32,
    },
}

impl Default for MeleeSwingType {
    fn default() -> Self {
        MeleeSwingType::stab()
    }
}

impl MeleeSwingType {
    pub fn stab() -> Self {
        MeleeSwingType::Stab { 
            speed: 10.0,             // Not used anymore but kept for compatibility
            duration: 0.15,  // Quick thrust forward
        }
    }

    pub fn slash() -> Self {
        MeleeSwingType::Slash {
            angular_speed: 12.0,     // Faster rotation (~2 full rotations/sec)
            radius: 50.0,
            current_angle: 0.0,
            duration: 0.4,           // Complete slash duration
            elapsed_time: 0.0,
        }
    }

    pub fn circle() -> Self {
        MeleeSwingType::Circle {
            expansion_speed: 100.0,
            current_radius: 0.0,
            duration: 0.5,
            elapsed_time: 0.0,
        }
    }

    pub fn get_total_duration(&self) -> f32 {
        match self {
            MeleeSwingType::Stab { duration, .. } => *duration,
            MeleeSwingType::Slash { duration, .. } => *duration,
            MeleeSwingType::Circle { duration, .. } => *duration,
        }
    }
}
#[derive(Bundle, Clone)]
pub struct MeleeSwingPropertiesBundle {
    pub damage: CollisionDamage,
    pub hitbox: MeleeHitbox,
    pub swing_type: MeleeSwingType,
    pub effects_list: EffectsList,
    pub sprite: Sprite,
}

#[derive(Component)]
pub struct MeleeSwingMarker;

#[derive(Component, Clone)]
#[require(
    LiveDuration, //Swing Time
    Sensor, 
    RigidBody(default_rigid_body),
    Collider,
    CollidingEntities,
    CollisionLayers(default_collision_layers)
)]
pub struct MeleeAttack{
    pub caster_entity: Entity,
}

fn default_rigid_body() -> RigidBody {
    RigidBody::Dynamic
}

fn default_collision_layers() -> CollisionLayers {
    // Currently projectiles can only collide with enemies
    CollisionLayers::new(
        GameCollisionLayer::InAir,
        [GameCollisionLayer::Enemy, GameCollisionLayer::HighObstacle],
    )
}
