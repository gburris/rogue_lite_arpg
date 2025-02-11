use std::{collections::HashMap, fs::File, io::BufReader};

use bevy::{prelude::Commands, scene::ron::de::from_reader};

use crate::map::{InstanceAssets, InstanceConfig, InstanceType};

pub fn setup_instance_data(mut commands: Commands) {
    let instance_config = load_instance_data("assets/config/instances.ron");
    commands.insert_resource(InstanceAssets { instance_config });
}

fn load_instance_data(path: &str) -> HashMap<String, InstanceType> {
    let file = File::open(path).expect("Failed to open RON file");
    let reader = BufReader::new(file);

    match from_reader::<_, InstanceConfig>(reader) {
        Ok(data) => data.instances,
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}
