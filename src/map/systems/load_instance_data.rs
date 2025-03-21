use std::{collections::HashMap, fs::File, io::BufReader};

use crate::map::components::{InstanceAssets, InstanceConfig, InstanceType};
use bevy::{prelude::Commands, scene::ron::de::from_reader};

pub fn setup_instance_data(mut commands: Commands) {
    let instance_config = load_instance_data();
    commands.insert_resource(InstanceAssets { instance_config });
}
fn fetch_instance_data() -> File {
    File::open("assets/config/instances.ron").expect("Failed to open RON file")
}

#[cfg(target_arch = "wasm32")]
fn fetch_instance_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/instances.ron")
}

fn load_instance_data() -> HashMap<String, InstanceType> {
    let reader = BufReader::new(fetch_instance_data());

    from_reader::<_, InstanceConfig>(reader)
        .unwrap_or_else(|e| {
            eprintln!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        })
        .instances
}
