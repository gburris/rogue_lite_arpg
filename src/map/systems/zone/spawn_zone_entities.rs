use bevy::prelude::*;
use rand::{Rng, rng};

use crate::{
    character::enemy::{EnemySpawnData, EnemyType},
    configuration::{ZLayer, assets::SpriteAssets},
    map::{
        chest::SpawnChestsEvent,
        components::{
            InstanceAssets, MapLayout, MarkerType, SpawnEnemies, SpawnNpcs, WorldSpaceConfig,
        },
        helpers::generator::generate_instance_layout,
        portal::Portal,
    },
    prelude::Player,
};

fn convert_tiles_to_world_positions(
    tile_positions: &[Vec2],
    world_config: &WorldSpaceConfig,
    map_layout: &MapLayout,
) -> Vec<Vec2> {
    tile_positions
        .iter()
        .map(|tile_position| world_config.tile_to_world(map_layout.size, tile_position.as_ivec2()))
        .collect()
}

pub fn spawn_zone_entities(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    instance_assets: Res<InstanceAssets>,
    player_query: Single<&mut Transform, With<Player>>,
) -> Result {
    //TODO: Markers should all store an associated type
    //So maps can have a set of enemy types that they create markers for
    //and chest types, and NPC types
    if let Some(exit_positions) = map_layout.markers.get_markers(MarkerType::LevelExits) {
        for exit_position in exit_positions {
            let exit_position_in_world =
                world_config.tile_to_world(map_layout.size, exit_position.as_ivec2());
            let portal_position = Vec3::new(
                exit_position_in_world.x,
                exit_position_in_world.y,
                ZLayer::OnGround.z(),
            );

            // Generate a unique instance layout for each portal
            let portal_instance = Portal {
                map_layout: generate_instance_layout(&instance_assets)?,
            };

            commands.spawn((
                portal_instance,
                Sprite::from_image(sprites.exit_door.clone()),
                Transform::from_translation(portal_position),
            ));
        }
    }

    if let Some(enemy_positions) = map_layout.markers.get_markers(MarkerType::EnemySpawns) {
        let spawn_positions =
            convert_tiles_to_world_positions(enemy_positions, &world_config, &map_layout);
        let mut rng = rng();
        let enemy_types = [EnemyType::FireMage, EnemyType::IceMage, EnemyType::Warrior];

        let enemy_spawn_data_list = spawn_positions
            .into_iter()
            .map(|pos| EnemySpawnData {
                position: pos,
                enemy_type: enemy_types[rng.random_range(0..3)].clone(),
            })
            .collect();

        commands.trigger(SpawnEnemies(enemy_spawn_data_list));
    }

    // Spawn chests
    if let Some(chest_positions) = map_layout.markers.get_markers(MarkerType::ChestSpawns) {
        let spawn_positions =
            convert_tiles_to_world_positions(chest_positions, &world_config, &map_layout);
        commands.trigger(SpawnChestsEvent(spawn_positions));
    }

    // Spawn NPCs
    if let Some(npc_positions) = map_layout.markers.get_markers(MarkerType::NPCSpawns) {
        let spawn_positions =
            convert_tiles_to_world_positions(npc_positions, &world_config, &map_layout);
        commands.trigger(SpawnNpcs(spawn_positions));
    }

    // Handle player spawn
    if let Some(spawn_positions) = map_layout.markers.get_markers(MarkerType::PlayerSpawns) {
        // Use first spawn position if multiple exist
        if let Some(spawn_position) = spawn_positions.first() {
            let player_spawn_position =
                world_config.tile_to_world(map_layout.size, spawn_position.as_ivec2());

            let mut player_transform = player_query.into_inner();
            player_transform.translation =
                player_spawn_position.extend(player_transform.translation.z);
        }
    } else {
        warn!("Player spawn marker not found in map layout.");
    }
    Ok(())
}
