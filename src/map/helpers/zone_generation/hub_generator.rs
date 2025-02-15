use std::collections::HashMap;

use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;

use crate::map::components::{MapLayout, MapMarkers, MarkerType, MultiMarkerType};

use super::{add_colliders::add_environmental_colliders_to_zone, create_tile_layout::create_hub};

pub fn generate_hub_layout() -> MapLayout {
    let size: TilemapSize = TilemapSize { x: 100, y: 100 };

    let hub_size = TilemapSize { x: 25, y: 25 };
    let tiles = create_hub(size, hub_size);

    let markers: MapMarkers = generate_hub_markers(size, hub_size);

    let environmental_colliders = add_environmental_colliders_to_zone(&tiles, size);

    MapLayout {
        size,
        tiles,
        markers,
        environmental_colliders,
    }
}

fn generate_hub_markers(map_size: TilemapSize, hub_size: TilemapSize) -> MapMarkers {
    let hub_center = Vec2::new((map_size.x / 2) as f32, (map_size.y / 2) as f32);
    let hub_bounds =
        Rect::from_center_size(hub_center, Vec2::new(hub_size.x as f32, hub_size.y as f32));
    let mut single_markers = HashMap::new();
    let mut multi_markers = HashMap::new();
    let center_of_hub = Vec2::new(
        (hub_bounds.min.x + hub_bounds.max.x) / 2.0,
        (hub_bounds.min.y + hub_bounds.max.y) / 2.0,
    );

    let player_spawn = Vec2::new(center_of_hub.x, hub_bounds.min.y as f32 + 5.0);
    single_markers.insert(MarkerType::PlayerSpawn, player_spawn);

    let level_exit_spawn = Vec2::new(center_of_hub.x, hub_bounds.min.y as f32 + 23.0);

    single_markers.insert(MarkerType::LevelExit, level_exit_spawn);
    let npc_positiions = vec![
        Vec2::new(center_of_hub.x + 5.0, center_of_hub.y + 5.0),
        Vec2::new(center_of_hub.x - 5.0, center_of_hub.y + -5.0),
        Vec2::new(center_of_hub.x + 5.0, center_of_hub.y + -5.0),
    ];
    multi_markers.insert(MultiMarkerType::NPCSpawns, npc_positiions);

    MapMarkers {
        single_markers,
        multi_markers,
    }
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }
}
