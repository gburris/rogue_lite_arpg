use std::{collections::HashMap, fs::File, io::BufReader};

use bevy::{prelude::Commands, scene::ron::de::from_reader};

use crate::enemy::{EnemiesConfig, EnemyAssets, EnemyType};

pub fn setup_enemy_assets(mut commands: Commands) {
    let enemy_config = load_enemy_data("assets/scenes/enemies.ron");
    commands.insert_resource(EnemyAssets { enemy_config });
}

fn load_enemy_data(path: &str) -> HashMap<String, EnemyType> {
    let file = File::open(path).expect("Failed to open RON file");
    let reader = BufReader::new(file);

    match from_reader::<_, EnemiesConfig>(reader) {
        Ok(data) => data
            .enemies
            .into_iter()
            .map(|enemy| (enemy.name.clone(), enemy))
            .collect(),
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}
