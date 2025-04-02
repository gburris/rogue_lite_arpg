use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    configuration::GameCollisionLayer,
    configuration::ZLayer,
    map::components::{EnvironmentalType, MapLayout, Wall, WorldSpaceConfig},
};

pub fn spawn_zone_colliders(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
) {
    let tile_size = world_config.tile_size;

    // Calculate center offset based on tilemap centering logic
    let grid_size = TilemapGridSize::new(tile_size.x, tile_size.y);
    let map_type = TilemapType::Square;

    let low = TilePos::new(0, 0).center_in_world(&grid_size, &map_type);
    let high =
        TilePos::new(map_layout.size.x, map_layout.size.y).center_in_world(&grid_size, &map_type);
    let diff = high - low;
    let offset = Vec2::new(-diff.x / 2.0, -diff.y / 2.0);

    // Spawn all environmental colliders
    for collider in &map_layout.environmental_colliders {
        // Convert tile position to world position
        let pos = Vec2::new(
            collider.transform.translation.x * tile_size.x,
            collider.transform.translation.y * tile_size.y,
        ) + offset;

        // Scale the collider based on tile size
        let scaled_collider =
            Collider::rectangle(collider.width * tile_size.x, collider.height * tile_size.y);

        let mut entity_commands = commands.spawn((
            RigidBody::Static,
            scaled_collider,
            Transform::from_xyz(pos.x, pos.y, ZLayer::OnGround.z()),
            GlobalTransform::default(),
        ));

        match collider.collider_type {
            EnvironmentalType::Wall => {
                entity_commands.insert((
                    Wall,
                    CollisionLayers::new(
                        GameCollisionLayer::HighObstacle,
                        GameCollisionLayer::HIGH_OBSTACLE_FILTERS,
                    ),
                ));
            }
            EnvironmentalType::Water => todo!(),
        }
    }
}
