use bevy::prelude::*;

use crate::map::helpers::zone_generation::generate_hub_layout;

pub fn insert_hub_layout(mut commands: Commands) {
    let map_layout = generate_hub_layout();
    commands.insert_resource(map_layout);
}
