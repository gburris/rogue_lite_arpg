use avian2d::prelude::{CollidingEntities, CollisionLayers, Sensor};
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

#[derive(Component, Default)]
#[require(CollidingEntities, Sensor)]
pub struct ProjectileReflection;

impl ProjectileReflection {
    pub fn collision_layers() -> CollisionLayers {
        //TODO: Shield PR:  Need a layer mask
        CollisionLayers::new(GameCollisionLayer::HighObstacle, GameCollisionLayer::InAir)
    }
}

#[derive(Component)]
pub struct ActiveShield;
