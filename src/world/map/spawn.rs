use std::{collections::HashMap, sync::OnceLock};

use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::RngExt;

use crate::{
    prelude::*,
    world::map::{
        MapCoordinates, TileType,
        blueprint::{MapBlueprint, PlacementKind},
        collisions::derive_collider_rects,
        walls::Wall,
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
        .add_observer(despawn_all::<CleanupZone, Wall>);
}

#[derive(Event)]
pub struct CleanupZone;

fn transition_to_playing(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::Playing);
}

fn spawn_zone_colliders(
    mut commands: Commands,
    blueprint: Res<MapBlueprint>,
    coordinates: Res<MapCoordinates>,
) {
    derive_collider_rects(&blueprint.tiles)
        .into_iter()
        .for_each(|rect| {
            let collider_pos = coordinates
                .tile_space_to_world_pos(blueprint.map_size().into(), rect.center_tile().as_vec2());

            let scaled_collider = Collider::rectangle(
                rect.size.x as f32 * coordinates.tile_size.x,
                rect.size.y as f32 * coordinates.tile_size.y,
            );

            commands.spawn((
                RigidBody::Static,
                scaled_collider,
                Transform::from_xyz(collider_pos.x, collider_pos.y, ZLayer::OnGround.z()),
                GlobalTransform::default(),
                Wall,
                CollisionLayers::new(
                    GameCollisionLayer::HighObstacle,
                    GameCollisionLayer::HIGH_OBSTACLE_FILTERS,
                ),
            ));
        });
}

fn spawn_zone_entities(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    blueprint: Res<MapBlueprint>,
    coordinates: Res<MapCoordinates>,
    player_query: Single<&mut Transform, With<Player>>,
) {
    let mut player_transform = player_query.into_inner();

    for placement in blueprint.placements.iter() {
        let world_spawn_pos =
            coordinates.tile_corner_to_world_pos(blueprint.map_size().into(), placement.tile);

        match &placement.kind {
            PlacementKind::PlayerStart => {
                info!("moving player");
                player_transform.translation =
                    world_spawn_pos.extend(player_transform.translation.z);
            }
            PlacementKind::Enemy(enemy_type) => {
                let enemy_spawn_data = EnemySpawnData {
                    position: world_spawn_pos,
                    enemy_type: enemy_type.clone(),
                };

                info!("spawning enemy");
                commands.trigger(SpawnEnemies(vec![enemy_spawn_data]));
            }
            PlacementKind::Npc(npc_type) => {
                info!("spawning npc");
                commands.trigger(SpawnNpcs(vec![NpcSpawnData {
                    position: world_spawn_pos,
                    npc_type: *npc_type,
                }]));
            }
            PlacementKind::Chest => {
                commands.trigger(SpawnChestsEvent(vec![world_spawn_pos]));
            }
            PlacementKind::Exit => {
                info!("spawning portal");
                commands.spawn(portal(&sprites, world_spawn_pos));
            }
        }
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
    blueprint: Res<MapBlueprint>,
    coodinates: Res<MapCoordinates>,
    sprites: Res<SpriteAssets>,
) {
    let map_size = blueprint.map_size();
    let tile_size = coodinates.tile_size;
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
        let storage = TileStorage::empty(map_size.into());
        storages.insert(*tile_type, (tilemap_entity, storage));
    }

    // Spawn tiles
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };

            if let Some(tile_type) = blueprint.tiles.get(IVec2::new(x as i32, y as i32))
                && let Some((tilemap_entity, storage)) = storages.get_mut(&tile_type)
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
                size: map_size.into(),
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
