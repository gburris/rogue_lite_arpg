use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{components::Health, movement::components::SimpleMotion};

//favoring #[require] as a default approach is generally recommended.
#[derive(Component)]
#[require(Health, SimpleMotion, CollisionDamage, Collider, Experience)]
pub struct Enemy;

//Experience granted by the enemy when player defeats it
#[derive(Component)]
pub struct Experience {
    pub base_exp: u32,
}

impl Default for Experience {
    fn default() -> Self {
        Experience { base_exp: 10 }
    }
}

//How much damage an enemy does when it collides with you
#[derive(Component, Clone)]
pub struct CollisionDamage {
    pub damage: f32,
}

impl Default for CollisionDamage {
    fn default() -> Self {
        CollisionDamage { damage: 10.1 }
    }
}
