use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    labels::layer::ZLayer,
    map::{portal::Portal, MapLayout, MarkerType, MultiMarkerType, WorldSpaceConfig},
    player::Player,
};

#[derive(Event)]
pub struct NPCSpawnEvent(pub Vec<Vec3>);
#[derive(Debug, Event)]
pub struct EnemySpawnEvent(pub Vec<Vec3>);

#[derive(Debug, Event)]
pub struct ChestSpawnEvent(pub Vec<Vec3>);

pub fn spawn_map_entities(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    // Spawn portal
    if let Some(exit_position_in_tiles) = map_layout.markers.get_single(MarkerType::LevelExit) {
        let exit_position_in_world =
            world_config.tile_to_world(map_layout.size, exit_position_in_tiles.as_ivec2());
        let portal_position = Vec3::new(
            exit_position_in_world.x,
            exit_position_in_world.y,
            ZLayer::Warpzone.z(),
        );

        commands.spawn((
            Portal,
            Sprite::from_image(sprites.exit_door.clone()),
            Transform::from_translation(portal_position),
        ));
    }

    // Spawn enemies if markers exist
    if let Some(enemy_positions) = map_layout.markers.get_multi(MultiMarkerType::EnemySpawns) {
        let spawn_positions = convert_tiles_to_world_positions(
            &enemy_positions,
            &world_config,
            &map_layout,
            ZLayer::Enemy,
        );
        commands.trigger(EnemySpawnEvent(spawn_positions));
    }

    // Spawn chests if markers exist
    if let Some(chest_positions) = map_layout.markers.get_multi(MultiMarkerType::ChestSpawns) {
        let spawn_positions = convert_tiles_to_world_positions(
            &chest_positions,
            &world_config,
            &map_layout,
            ZLayer::Enemy,
        );
        commands.trigger(ChestSpawnEvent(spawn_positions));
    }

    // Spawn NPCs if markers exist
    if let Some(npc_positions) = map_layout.markers.get_multi(MultiMarkerType::NPCSpawns) {
        let spawn_positions = convert_tiles_to_world_positions(
            &npc_positions,
            &world_config,
            &map_layout,
            ZLayer::Enemy,
        );
        commands.trigger(NPCSpawnEvent(spawn_positions));
    }

    // Handle player spawn
    if let Some(spawn_position_in_tiles) = map_layout.markers.get_single(MarkerType::PlayerSpawn) {
        let spawn_position_in_world =
            world_config.tile_to_world(map_layout.size, spawn_position_in_tiles.as_ivec2());
        let player_spawn_position = Vec3::new(
            spawn_position_in_world.x,
            spawn_position_in_world.y,
            ZLayer::Player.z(),
        );

        if let Ok(mut player_transform) = query.get_single_mut() {
            player_transform.translation = player_spawn_position;
        } else {
            warn!("Player entity not found. Ensure the player is spawned before this system runs.");
        }
    } else {
        warn!("Player spawn marker not found in map layout.");
    }
}

fn convert_tiles_to_world_positions(
    tile_positions: &Vec<Vec2>,
    world_config: &WorldSpaceConfig,
    map_layout: &MapLayout,
    z_layer: ZLayer,
) -> Vec<Vec3> {
    tile_positions
        .iter()
        .map(|tile_position| {
            let world_position =
                world_config.tile_to_world(map_layout.size, tile_position.as_ivec2());
            Vec3::new(world_position.x, world_position.y, z_layer.z())
        })
        .collect()
}
