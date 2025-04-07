use bevy::{
    prelude::*,
    scene::ron::{self},
};
use serde::Deserialize;

use crate::{
    enemy::{EnemyAssets, Experience},
    map::components::InstanceAssets,
    player::PlayerStats,
    progression::GameProgress,
};

pub struct PropertiesPlugin;

impl Plugin for PropertiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_ron::<InstanceAssets>(include_bytes!("properties/instances.ron"))
            .insert_ron::<EnemyAssets>(include_bytes!("properties/enemies.ron"));
    }
}

impl Default for GameProgress {
    fn default() -> Self {
        GameProgress {
            game_completed_counter: 0,
            death_counter: 0,
            total_career_level: 0,
            progress_points: 5,
            base_stats: PlayerStats::default(),
        }
    }
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            agility: 1,
            strength: 1,
            dexterity: 1,
            intellect: 1,
            luck: 99,
        }
    }
}

impl Default for Experience {
    fn default() -> Self {
        Experience { base_exp: 10.0 }
    }
}

trait RonResourceExt {
    fn insert_ron<'de, T>(&mut self, data: &'static [u8]) -> &mut Self
    where
        T: Deserialize<'de> + Resource;
}
impl RonResourceExt for App {
    fn insert_ron<'de, T>(&mut self, data: &'static [u8]) -> &mut Self
    where
        T: Deserialize<'de> + Resource,
    {
        let data =
            ron::de::from_bytes::<T>(data).expect("failed to load properties from provided path");
        self.insert_resource(data);
        self
    }
}
