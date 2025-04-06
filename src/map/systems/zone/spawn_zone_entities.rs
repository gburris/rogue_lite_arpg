use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    configuration::{assets::SpriteAssets, ZLayer}, enemy::systems::enemy_spawn::{EnemySpawnData, EnemyType}, map::{
        chest::SpawnChestsEvent,
        components::{
            EnemiesSpawnEvent, InstanceAssets, MapLayout, MarkerType, NPCSpawnEvent,
            WorldSpaceConfig,
        },
        helpers::generator::generate_instance_layout,
        portal::Portal,
    }, npc::components::NPCSpawnData, player::Player
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
) {
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
                map_layout: generate_instance_layout(&instance_assets),
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
        let mut rng = thread_rng();

        let default_enemy_types = [EnemyType::FireMage, EnemyType::IceMage, EnemyType::Warrior];
        warn!(
            "Setting allowed types to 2 {:?}",
            map_layout.valid_enemy_types
        );
        let enemy_spawn_data_list: Vec<EnemySpawnData> = spawn_positions
            .into_iter()
            .map(|pos| {
                let enemy_type = match &map_layout.valid_enemy_types {
                    Some(types) if !types.is_empty() => {
                        // Use the provided types if available and non-empty
                        let index = rng.gen_range(0..types.len());
                        types[index].clone()
                    }
                    _ => {
                        // Fall back to defaults if None or empty
                        let index = rng.gen_range(0..default_enemy_types.len());
                        default_enemy_types[index].clone()
                    }
                };

                EnemySpawnData {
                    position: pos,
                    enemy_type,
                }
            })
            .collect();

        commands.trigger(EnemiesSpawnEvent(enemy_spawn_data_list));
    }

    // Spawn chests
    if let Some(chest_positions) = map_layout.markers.get_markers(MarkerType::ChestSpawns) {
        let spawn_positions =
            convert_tiles_to_world_positions(chest_positions, &world_config, &map_layout);
        commands.trigger(SpawnChestsEvent(spawn_positions));
    }

    if let Some(npc_positions) = map_layout.markers.get_markers(MarkerType::NPCSpawns) {
        let spawn_positions =
            convert_tiles_to_world_positions(npc_positions, &world_config, &map_layout);

        // Only spawn NPCs if valid types are provided and not empty
        if let Some(types) = &map_layout.valid_npc_types {
            if !types.is_empty() {
                let npc_spawn_data_list = spawn_positions
                    .into_iter()
                    .enumerate()
                    .map(|(index, pos)| {
                        // Use modulo to cycle through the available types
                        let npc_type = types[index % types.len()].clone();
                        NPCSpawnData {
                            position: pos,
                            npc_type,
                        }
                    })
                    .collect();

                commands.trigger(NPCSpawnEvent(npc_spawn_data_list));
            } else {
                warn!("No NPCs will spawn: valid_npc_types is empty.");
            }
        } else {
            warn!("No NPCs will spawn: valid_npc_types is None.");
        }
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
}
