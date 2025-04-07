use std::collections::HashMap;

use bevy::{
    prelude::*,
    scene::ron::{self},
};
use serde::Deserialize;

use crate::{
    enemy::{EnemyAssets, EnemyDetails},
    map::components::{InstanceAssets, InstanceType},
};

pub struct PropertiesPlugin;

impl Plugin for PropertiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InstanceAssets>()
            .init_resource::<EnemyAssets>();
    }
}

#[derive(Deserialize, Debug)]
pub struct EnemiesConfig {
    pub enemies: HashMap<String, EnemyDetails>,
}
fn load_enemy_data() -> EnemiesConfig {
    ron::de::from_bytes(include_bytes!("properties/enemies.ron"))
        .expect("Failed to load enemy properties from provided path")
}
impl Default for EnemyAssets {
    fn default() -> Self {
        let enemy_config: EnemiesConfig = load_enemy_data();
        Self {
            enemy_config: enemy_config.enemies,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct InstanceConfig {
    pub instances: HashMap<String, InstanceType>,
}
impl Default for InstanceAssets {
    fn default() -> Self {
        let instance_config: InstanceConfig = load_instance_config();
        Self {
            instance_config: instance_config.instances,
        }
    }
}
fn load_instance_config() -> InstanceConfig {
    ron::de::from_bytes(include_bytes!("properties/instances.ron"))
        .expect("Failed to load enemy properties from provided path")
}
