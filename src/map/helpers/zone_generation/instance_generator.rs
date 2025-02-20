use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

use crate::map::components::{InstanceAssets, MapLayout, MapMarkers};

use super::map_data::{MapDataBuilder, MarkerPlacement, Prefab};

pub fn generate_instance_layout(instance_assets: &Res<InstanceAssets>) -> MapLayout {
    let mut rng = rand::thread_rng();

    // Randomly select instance type & Resolves ranges from RON file
    let instance_type = if rng.gen_bool(0.9) {
        instance_assets.instance_config.get("Swamp").unwrap()
    } else {
        instance_assets.instance_config.get("TreasureRoom").unwrap()
    };
    let size_x = rng.gen_range(instance_type.size_x_range.0..=instance_type.size_x_range.1) as u32;
    let size_y = rng.gen_range(instance_type.size_y_range.0..=instance_type.size_y_range.1) as u32;
    let map_size = TilemapSize {
        x: size_x,
        y: size_y,
    };
    let num_enemies = rng.gen_range(
        instance_type.number_of_enemies_range.0..=instance_type.number_of_enemies_range.1,
    ) as u32;
    let num_chests =
        rng.gen_range(instance_type.chest_range.0..=instance_type.chest_range.1) as u32;

    let map_data = MapDataBuilder::new(map_size)
        .with_dead_zones(instance_type.dead_zone_squares)
        .with_prefab(Prefab::Temple)
        .with_exterior_walls()
        .with_chests(num_chests)
        .with_enemies(num_enemies)
        .with_marker_placement(MarkerPlacement::Random)
        .build();

    MapLayout {
        size: map_data.size,
        tiles: map_data.tiles,
        markers: MapMarkers {
            markers: map_data.markers,
        },
        environmental_colliders: map_data.colliders,
    }
}
