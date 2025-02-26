use crate::enemy::{EnemiesConfig, EnemyAssets, EnemyDetails};
use bevy::{prelude::Commands, scene::ron::de::from_reader};
use std::{collections::HashMap, fs::File, io::BufReader};

#[cfg(target_arch = "wasm32")]
use bevy::scene::ron::from_str;

pub fn setup_enemy_assets(mut commands: Commands) {
    let enemy_config = load_enemy_data();
    commands.insert_resource(EnemyAssets { enemy_config });
}

#[cfg(not(target_arch = "wasm32"))]
fn load_enemy_data() -> HashMap<String, EnemyDetails> {
    let file = File::open("assets/config/enemies.ron").expect("Failed to open RON file");
    let reader = BufReader::new(file);

    match from_reader::<_, EnemiesConfig>(reader) {
        Ok(data) => data.enemies,
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_enemy_data() -> HashMap<String, EnemyDetails> {
    const ENEMY_RON: &str = include_str!("../../../assets/config/enemies.ron");

    match from_str::<EnemiesConfig>(ENEMY_RON) {
        Ok(data) => data.enemies,
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}
