use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    labels::layer::ZLayer,
    map::{portal::Portal, MapLayout, MarkerType, MultiMarkerType, WorldSpaceConfig},
    player::Player,
};

#[derive(Event)]
pub struct NPCSpawnEvent(pub Vec<Vec3>);

pub fn spawn_hub_entities(
    mut commands: Commands,
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
    let warp_position: Vec3 = Vec3::new(
        exit_position_in_world.x,
        exit_position_in_world.y,
        ZLayer::Warpzone.z(),
    );

    commands.spawn((
        Portal::StartingPortal,
        Sprite::from_image(sprites.run_start_door.clone()),
        Transform::from_translation(warp_position),
    ));

    if let Some(npc_spawn_positions_in_tiles) =
        map_layout.markers.get_multi(MultiMarkerType::NPCSpawns)
    {
        let npc_spawn_positions: Vec<Vec3> = npc_spawn_positions_in_tiles
            .iter()
            .map(|tile_position| {
                let world_position =
                    world_config.tile_to_world(map_layout.size, tile_position.as_ivec2());
                Vec3::new(world_position.x, world_position.y, ZLayer::Enemy.z())
            })
            .collect();
        commands.trigger(NPCSpawnEvent(npc_spawn_positions));
    } else {
        warn!("No NPC spawn markers found in map layout.");
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
}
