use avian2d::prelude::{CollidingEntities, CollisionLayers, LayerMask, Sensor};
use bevy::{prelude::*, utils::{HashMap, HashSet}};

use crate::configuration::GameCollisionLayer;

#[derive(Component, Default)]
#[require(CollidingEntities, Sensor)]
pub struct ProjectileReflection;

impl ProjectileReflection {
    pub fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HighObstacle, GameCollisionLayer::InAir)
    }
}

#[derive(Component)]
pub struct ActiveShield {
    pub projectiles_reflected: HashSet<Entity>,
}
