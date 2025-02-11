use std::{collections::HashMap, fs::File, io::BufReader};

use bevy::{prelude::Commands, scene::ron::de::from_reader};

use crate::map::{InstanceAssets, InstanceConfig, InstanceType};

pub fn setup_instance_data(mut commands: Commands) {
    let instance_config = load_instance_data();
    commands.insert_resource(InstanceAssets { instance_config });
}

#[cfg(not(target_arch = "wasm32"))]
fn load_instance_data() -> HashMap<String, InstanceType> {
    let file = File::open("assets/config/instances.ron").expect("Failed to open RON file");
    let reader = BufReader::new(file);

    match from_reader::<_, InstanceConfig>(reader) {
        Ok(data) => data.instances,
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_instance_data() -> HashMap<String, InstanceType> {
    const INSTANCE_RON: &str = include_str!("../../../../assets/config/instances.ron");

    match from_str::<InstanceConfig>(INSTANCE_RON) {
        Ok(data) => data.instances,
        Err(e) => {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        }
    }
}
