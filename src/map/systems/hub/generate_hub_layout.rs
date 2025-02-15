use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::helpers::hub_map_layout::{self};

pub fn generate_hub_layout(mut commands: Commands) {
    //HUB has a fixed size
    let map_layout = hub_map_layout::generate_hub_map(TilemapSize { x: 100, y: 100 });
    commands.insert_resource(map_layout);
}
