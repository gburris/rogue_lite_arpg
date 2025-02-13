use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    labels::layer::ZLayer,
    map::{
        portal::Portal, resources::CurrentZoneLevel, MapLayout, MarkerType, MultiMarkerType,
        WorldSpaceConfig,
    },
    player::Player,
};

#[derive(Debug, Event)]
pub struct EnemySpawnEvent(pub Vec<Vec3>);

#[derive(Debug, Event)]
pub struct ChestSpawnEvent(pub Vec<Vec3>);

pub fn spawn_instance_entities(
    mut commands: Commands,
    mut zone_level: ResMut<CurrentZoneLevel>,
    sprites: Res<SpriteAssets>,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    mut query: Query<&mut Transform, With<Player>>, // Query to locate the player entity
) {
    // Find exit position and spawn exit portal
    let exit_position_in_tiles = map_layout
        .markers
        .get_single(MarkerType::LevelExit)
        .unwrap();
    let exit_position_in_world =
        world_config.tile_to_world(map_layout.size, exit_position_in_tiles.as_ivec2());
    let warpzone_position: Vec3 = Vec3::new(
        exit_position_in_world.x,
        exit_position_in_world.y,
        ZLayer::Warpzone.z(),
    );

    commands.spawn((
        Portal::WarpZone,
        Sprite::from_image(sprites.warp_zone.clone()),
        Transform::from_translation(warpzone_position),
    ));

    if let Some(enemy_spawn_positions_in_tiles) =
        map_layout.markers.get_multi(MultiMarkerType::EnemySpawns)
    {
        let enemy_spawn_positions: Vec<Vec3> = enemy_spawn_positions_in_tiles
            .iter()
            .map(|tile_position| {
                let world_position =
                    world_config.tile_to_world(map_layout.size, tile_position.as_ivec2());
                Vec3::new(world_position.x, world_position.y, ZLayer::Enemy.z())
            })
            .collect();
        commands.trigger(EnemySpawnEvent(enemy_spawn_positions));
    } else {
        warn!("No enemy spawn markers found in map layout.");
    }

    if let Some(chest_spawn_pos_in_tiles) =
        map_layout.markers.get_multi(MultiMarkerType::ChestSpawns)
    {
        let chest_spawn_positions: Vec<Vec3> = chest_spawn_pos_in_tiles
            .iter()
            .map(|tile_position| {
                let world_position =
                    world_config.tile_to_world(map_layout.size, tile_position.as_ivec2());
                Vec3::new(world_position.x, world_position.y, ZLayer::Enemy.z())
            })
            .collect();
        commands.trigger(ChestSpawnEvent(chest_spawn_positions));
    } else {
        warn!("No chest spawn markers found in map layout.");
    }

    // Locate the player spawn position
    if let Some(spawn_position_in_tiles) = map_layout.markers.get_single(MarkerType::PlayerSpawn) {
        let spawn_position_in_world =
            world_config.tile_to_world(map_layout.size, spawn_position_in_tiles.as_ivec2());
        let player_spawn_position: Vec3 = Vec3::new(
            spawn_position_in_world.x,
            spawn_position_in_world.y,
            ZLayer::Player.z(),
        );

        // Update the player's position
        if let Ok(mut player_transform) = query.get_single_mut() {
            player_transform.translation = player_spawn_position;
        } else {
            warn!("Player entity not found. Ensure the player is spawned before this system runs.");
        }
    } else {
        warn!("Player spawn marker not found in map layout.");
    }

    // Increment zone level
    zone_level.0 += 1;
}
