use std::{collections::HashMap, sync::OnceLock};

use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, rng};

use crate::{
    character::{enemy::SpawnEnemies, npc::SpawnNpcs},
    prelude::*,
    world::map::{
        EnvironmentalType, MapLayout, MarkerType, TileType, WorldSpaceConfig, walls::Wall,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::SpawnZone),
        (
            spawn_zone_tilemap,
            spawn_zone_colliders,
            spawn_zone_entities,
            transition_to_playing,
        )
            .chain(),
    );

    app.add_observer(despawn_all::<CleanupZone, TilemapId>)
        .add_observer(despawn_all::<CleanupZone, Wall>)
        .add_observer(despawn_all::<CleanupZone, Water>);
}

#[derive(Event)]
pub struct CleanupZone;

fn transition_to_playing(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::Playing);
}

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

fn spawn_zone_colliders(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
) {
    let tile_size = world_config.tile_size;

    // Calculate center offset based on tilemap centering logic
    let grid_size = TilemapGridSize::new(tile_size.x, tile_size.y);
    let map_type = TilemapType::Square;

    let low = TilePos::new(0, 0).center_in_world(
        &map_layout.size,
        &grid_size,
        &tile_size,
        &map_type,
        &TilemapAnchor::Center,
    );
    let high = TilePos::new(map_layout.size.x, map_layout.size.y).center_in_world(
        &map_layout.size,
        &grid_size,
        &tile_size,
        &map_type,
        &TilemapAnchor::Center,
    );
    let diff = high - low;
    let offset = Vec2::new(-diff.x / 2.0, -diff.y / 2.0);

    // Spawn all environmental colliders
    for collider in &map_layout.environmental_colliders {
        // Convert tile position to world position
        let pos: Vec2 = Vec2::new(
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
                info!("spawning wall");
            }
            EnvironmentalType::Water => todo!(),
        }
    }
}

fn spawn_zone_entities(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    player_query: Single<&mut Transform, With<Player>>,
) {
    //TODO: Markers should all store an associated type
    //So maps can have a set of enemy types that they create markers for
    //and chest types, and NPC types
    if let Some(exit_positions) = map_layout.markers.get_markers(MarkerType::LevelExits) {
        for exit_position in exit_positions {
            let exit_position_in_world =
                world_config.tile_to_world(map_layout.size, exit_position.as_ivec2());
            info!("spawning portal");
            commands.spawn(portal(&sprites, exit_position_in_world));
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

        info!("spawning enemies");
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

            info!("moving player");

            player_transform.translation =
                player_spawn_position.extend(player_transform.translation.z);
        }
    } else {
        warn!("Player spawn marker not found in map layout.");
    }
}

#[derive(Clone, Copy)]
enum TileIndexType {
    Random(u32), // Maximum random value
    Fixed(u32),  // Fixed index
}

fn tile_configurations() -> &'static HashMap<TileType, TileIndexType> {
    static CONFIGS: OnceLock<HashMap<TileType, TileIndexType>> = OnceLock::new();
    CONFIGS.get_or_init(|| {
        let mut m = HashMap::new();

        m.insert(TileType::Ground, TileIndexType::Random(10));
        m.insert(TileType::Grass, TileIndexType::Random(10));
        m.insert(TileType::Wall, TileIndexType::Fixed(0));
        m.insert(TileType::Water, TileIndexType::Fixed(0));
        m.insert(TileType::Wood, TileIndexType::Random(10));
        m.insert(TileType::Cobblestone, TileIndexType::Random(10));

        m
    })
}

fn spawn_zone_tilemap(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
    world_config: Res<WorldSpaceConfig>,
    sprites: Res<SpriteAssets>,
) {
    let map_size = map_layout.size;
    let tile_size = world_config.tile_size;
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::Square;

    // Create storage and entities for each tile type
    let mut storages: HashMap<TileType, (Entity, TileStorage)> = HashMap::new();

    // Get texture handles based on tile type
    let texture_handles: HashMap<TileType, Handle<Image>> = HashMap::from([
        (TileType::Ground, sprites.ground_tiles.clone()),
        (TileType::Grass, sprites.grass_tiles.clone()),
        (TileType::Wall, sprites.wall_tiles.clone()),
        (TileType::Water, sprites.water_tiles.clone()),
        (TileType::Wood, sprites.wood_tiles.clone()),
        (TileType::Cobblestone, sprites.cobblestone_tiles.clone()),
    ]);

    // Initialize storage for each tile type
    for tile_type in tile_configurations().keys() {
        let tilemap_entity = commands.spawn_empty().id();
        let storage = TileStorage::empty(map_size);
        storages.insert(*tile_type, (tilemap_entity, storage));
    }

    // Spawn tiles
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_type = map_layout.tiles[x as usize][y as usize];

            if let Some((tilemap_entity, storage)) = storages.get_mut(&tile_type)
                && let Some(index_type) = tile_configurations().get(&tile_type)
            {
                let texture_index = match index_type {
                    TileIndexType::Random(max) => rand::rng().random_range(0..*max),
                    TileIndexType::Fixed(index) => *index,
                };

                let tile_entity = commands
                    .spawn((
                        Name::new("Tile"),
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(*tilemap_entity),
                            texture_index: TileTextureIndex(texture_index),
                            ..default()
                        },
                    ))
                    .id();
                storage.set(&tile_pos, tile_entity);
            }
        }
    }

    // Insert tilemaps
    for (tile_type, (entity, storage)) in storages {
        if let Some(texture_handle) = texture_handles.get(&tile_type) {
            commands.entity(entity).insert(TilemapBundle {
                grid_size,
                size: map_size,
                storage,
                map_type,
                texture: TilemapTexture::Single(texture_handle.clone()),
                tile_size,
                anchor: TilemapAnchor::Center,
                transform: Transform::from_xyz(0.0, 0.0, ZLayer::Ground.z()),
                ..default()
            });
        }
    }
}
