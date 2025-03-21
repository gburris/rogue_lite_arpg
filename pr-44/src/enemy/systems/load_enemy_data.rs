use crate::enemy::{EnemiesConfig, EnemyAssets, EnemyDetails};
use bevy::{prelude::Commands, scene::ron::de::from_reader};
use std::{collections::HashMap, fs::File, io::BufReader};

pub fn setup_enemy_assets(mut commands: Commands) {
    let enemy_config = load_enemy_data();
    commands.insert_resource(EnemyAssets { enemy_config });
}

#[cfg(not(target_arch = "wasm32"))]
fn fetch_enemy_data() -> File {
    File::open("assets/config/enemies.ron").expect("Failed to open RON file")
}

#[cfg(target_arch = "wasm32")]
fn fetch_enemy_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/enemies.ron")
}

fn load_enemy_data() -> HashMap<String, EnemyDetails> {
    let reader = BufReader::new(fetch_enemy_data());

    match from_reader::<_, EnemiesConfig>(reader) {
        Ok(data) => data.enemies,
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}
