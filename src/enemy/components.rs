use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::character::Character;

#[derive(Component)]
#[require(Character, Experience)]
pub struct Enemy;

//Experience granted by the enemy when player defeats it
#[derive(Component)]
pub struct Experience {
    pub base_exp: f32,
}

impl Default for Experience {
    fn default() -> Self {
        Experience { base_exp: 10.0 }
    }
}

#[derive(Deserialize, Debug)]
pub struct EnemiesConfig {
    pub enemies: HashMap<String, EnemyDetails>,
}
#[derive(Deserialize, Debug)]
pub struct EnemyDetails {
    pub simple_motion_speed: f32,
    pub health: f32,
    pub sprite_path: String,
    pub collider_size: (f32, f32),
    pub weapon: String,
}

#[derive(Resource)]
pub struct EnemyAssets {
    pub enemy_config: HashMap<String, EnemyDetails>,
}
