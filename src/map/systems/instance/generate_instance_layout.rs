//This will call the helper
//Generate generate_map_layout
//And then store it as a resource
use bevy::prelude::*;

use crate::map::{helpers::map_layout::generate_map_layout, WorldSpaceConfig};

pub fn generate_instance_layout(mut commands: Commands, world_config: Res<WorldSpaceConfig>) {
    let map_layout = generate_map_layout(world_config.map_size);
    commands.insert_resource(map_layout);
}
