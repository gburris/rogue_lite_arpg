use bevy::{
    prelude::*,
    scene::ron::{self},
};
use serde::Deserialize;

use crate::{
    ai::SimpleMotion,
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

impl Default for SimpleMotion {
    fn default() -> Self {
        SimpleMotion::new(10.0)
    }
}

/// This trait is used to load properties from a RON file.
/// The RON file is included in the binary using the `include_bytes!` macro.
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

// NOTE: Alternative way to load from RON is to have `impl RonData` for each struct
// and use `ron::de::from_bytes::<T>(T::DATA)` instead of `include_bytes!`.
#[allow(dead_code)]
trait RonData {
    #[allow(non_snake_case)]
    const DATA: &'static [u8];
}

#[allow(dead_code)]
trait InitRonResourceExt {
    fn init_ron<'de, T>(&mut self) -> &mut Self
    where
        T: Deserialize<'de> + Resource + RonData;
}

impl InitRonResourceExt for App {
    fn init_ron<'de, T>(&mut self) -> &mut Self
    where
        T: Deserialize<'de> + Resource + RonData,
    {
        let data = ron::de::from_bytes::<T>(T::DATA)
            .expect("failed to load properties from provided path");
        self.insert_resource(data);
        self
    }
}
