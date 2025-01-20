use std::collections::HashMap;

use avian2d::prelude::Collider;
use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    combat::damage::components::{CollisionDamage, Health},
    movement::components::SimpleMotion,
};

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

#[derive(Deserialize, Debug)]
pub struct EnemiesConfig {
    pub enemies: Vec<EnemyType>,
}
#[derive(Deserialize, Debug)]
pub struct EnemyType {
    pub name: String,
    pub simple_motion_speed: f32,
    pub health: f32,
    pub sprite_path: String,
    pub collider_size: (f32, f32),
}

#[derive(Resource)]
pub struct EnemyAssets {
    pub enemy_config: HashMap<String, EnemyType>,
}
