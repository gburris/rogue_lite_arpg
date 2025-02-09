use bevy::prelude::*;

use crate::map::{
    helpers::hub_map_layout::{self},
    WorldSpaceConfig,
};

pub fn generate_hub_layout(mut commands: Commands, world_config: Res<WorldSpaceConfig>) {
    let map_layout = hub_map_layout::generate_hub_map(world_config.map_size);
    commands.insert_resource(map_layout);
}
