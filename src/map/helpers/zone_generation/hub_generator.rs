use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use std::collections::HashMap;

use super::{create_tile_layout::create_hub, utils::calculate_center_rect};
use crate::map::components::{MapLayout, MapMarkers, MarkerType};

const PLAYER_SPAWN_Y_OFFSET: f32 = 5.0;
const LEVEL_EXIT_Y_OFFSET: f32 = 23.0;
const NPC_OFFSET: f32 = 5.0;

pub fn generate_hub_layout() -> MapLayout {
    let size = TilemapSize { x: 100, y: 100 };
    let hub_size = TilemapSize { x: 25, y: 25 };

    let map_data = create_hub(size, hub_size);
    let markers = generate_hub_markers(size, hub_size);
    // let environmental_colliders = add_environmental_colliders_to_zone(&tiles, size);

    MapLayout {
        size,
        tiles: map_data.tiles,
        markers,
        environmental_colliders: map_data.colliders,
    }
}

fn generate_hub_markers(map_size: TilemapSize, hub_size: TilemapSize) -> MapMarkers {
    let mut markers = HashMap::new();

    let hub_bounds = calculate_center_rect(map_size, hub_size);

    let center_of_hub = hub_bounds.center();

    // Generate player spawn
    let player_spawn = Vec2::new(center_of_hub.x, hub_bounds.min.y + PLAYER_SPAWN_Y_OFFSET);
    markers.insert(MarkerType::PlayerSpawns, vec![player_spawn]);

    // Generate level exit
    let level_exit = Vec2::new(center_of_hub.x, hub_bounds.min.y + LEVEL_EXIT_Y_OFFSET);
    markers.insert(MarkerType::LevelExits, vec![level_exit]);

    // Generate NPC positions
    let npc_positions = vec![
        Vec2::new(center_of_hub.x + NPC_OFFSET, center_of_hub.y + NPC_OFFSET),
        Vec2::new(center_of_hub.x - NPC_OFFSET, center_of_hub.y - NPC_OFFSET),
        Vec2::new(center_of_hub.x + NPC_OFFSET, center_of_hub.y - NPC_OFFSET),
    ];
    markers.insert(MarkerType::NPCSpawns, npc_positions);

    MapMarkers { markers }
}
