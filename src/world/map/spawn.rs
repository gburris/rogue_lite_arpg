use std::collections::BTreeMap;
use std::{collections::HashMap, sync::OnceLock};

use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use rand::RngExt;

use crate::{
    prelude::*,
    world::map::blueprint::{MapBlueprint, PlacementKind},
};

use bevy_ecs_tilemap::map::TilemapTileSize;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::SpawnMap),
        (
            spawn_map_tiles,
            spawn_map_colliders,
            spawn_map_entities,
            transition_to_playing,
        )
            .chain(),
    );

    app.insert_resource(MapCoordinates::default());
}

fn transition_to_playing(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::Playing);
}

fn spawn_map_tiles(
    mut commands: Commands,
    blueprint: Single<(&MapBlueprint, Entity)>,
    mut coordinates: ResMut<MapCoordinates>,
    sprites: Res<SpriteAssets>,
) {
    let (blueprint, blueprint_entity) = blueprint.into_inner();
    let map_size = blueprint.map_size();
    coordinates.map_size = map_size;
    let tile_size = coordinates.tile_size;
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
                        ChildOf(blueprint_entity),
                    ))
                    .id();
                storage.set(&tile_pos, tile_entity);
            }
        }
    }

    // Insert tilemaps
    for (tile_type, (entity, storage)) in storages {
        if let Some(texture_handle) = texture_handles.get(&tile_type) {
            commands.entity(entity).insert((
                TilemapBundle {
                    grid_size,
                    size: map_size.into(),
                    storage,
                    map_type,
                    texture: TilemapTexture::Single(texture_handle.clone()),
                    tile_size,
                    anchor: TilemapAnchor::Center,
                    transform: Transform::from_xyz(0.0, 0.0, ZLayer::Ground.z()),

                    ..default()
                },
                ChildOf(blueprint_entity),
            ));
        }
    }
}

#[derive(Component)]
#[require(DespawnOnExit::<AppState>(AppState::Playing))]
pub struct MapCollider;

fn spawn_map_colliders(
    mut commands: Commands,
    blueprint: Single<&MapBlueprint>,
    coordinates: Res<MapCoordinates>,
) {
    for rect in derive_collider_rects(&blueprint.tiles) {
        let (position, size) = coordinates.tile_rect_to_world_rect(blueprint.map_size(), rect);

        commands.spawn((
            RigidBody::Static,
            Collider::rectangle(size.x, size.y),
            Transform::from_xyz(position.x, position.y, ZLayer::OnGround.z()),
            GlobalTransform::default(),
            MapCollider,
            CollisionLayers::new(
                GameCollisionLayer::HighObstacle,
                GameCollisionLayer::HIGH_OBSTACLE_FILTERS,
            ),
        ));
    }
}

fn derive_collider_rects(tiles: &TileGrid) -> Vec<TileRect> {
    let map_width = i32::try_from(tiles.map_size().x).expect("map width fits in i32");
    let map_height = i32::try_from(tiles.map_size().y).expect("map height fits in i32");

    let mut completed_collider_rects = Vec::new();
    let mut previous_runs: BTreeMap<(i32, i32), TileRect> = BTreeMap::new();

    for y in 0..map_height {
        let runs = blocking_runs(tiles, y, map_width);
        let mut continued_runs = BTreeMap::new();

        for (start_x, end_x) in runs {
            let tree_key = (start_x, end_x);

            let rect = if let Some(mut rect) = previous_runs.remove(&tree_key) {
                rect.size.y += 1;
                rect
            } else {
                TileRect {
                    min: IVec2::new(start_x, y),
                    size: UVec2::new((end_x - start_x) as u32, 1),
                }
            };

            continued_runs.insert(tree_key, rect);
        }

        // Any rectangles that remain in previous_runs did not have a matching run
        // in the current row, so they are complete and can be added to the final list.
        completed_collider_rects.extend(previous_runs.into_values());
        previous_runs = continued_runs;
    }

    completed_collider_rects.extend(previous_runs.into_values());
    completed_collider_rects
}

fn blocking_runs(tiles: &TileGrid, y: i32, map_width: i32) -> Vec<(i32, i32)> {
    let mut runs = Vec::new();
    let mut x = 0;

    while x < map_width {
        while x < map_width
            && !tiles
                .get(IVec2::new(x, y))
                .is_some_and(|tile_type| tile_type.blocks_ground())
        {
            x += 1;
        }

        let start = x;

        while x < map_width
            && tiles
                .get(IVec2::new(x, y))
                .is_some_and(|tile_type| tile_type.blocks_ground())
        {
            x += 1;
        }

        if start < x {
            runs.push((start, x));
        }
    }

    runs
}

fn spawn_map_entities(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    blueprint: Single<&MapBlueprint>,
    coordinates: Res<MapCoordinates>,
    player_query: Single<&mut Transform, With<Player>>,
) {
    let mut player_transform = player_query.into_inner();

    for placement in &blueprint.placements {
        let world_spawn_pos =
            coordinates.tile_center_to_world_pos(blueprint.map_size(), placement.tile);

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

//This holds the concept of "Tiles are this big relative to world cordinaties"
#[derive(Resource)]
pub struct MapCoordinates {
    pub tile_size: TilemapTileSize,
    pub world_origin: Vec2,
    pub map_size: UVec2,
}

//If we want to f with the scale of our tiles:world (e.g. have way more tiles in our world)
//We can edit that here
impl Default for MapCoordinates {
    fn default() -> Self {
        MapCoordinates {
            tile_size: TilemapTileSize::new(TILE_SIZE, TILE_SIZE),
            world_origin: Vec2::ZERO,
            map_size: UVec2::ZERO,
        }
    }
}

//Helper impl -> Pass in a tile, and it tells you the world co-ords it maps to
//This seems jank, but it's because the rendering of the tiles has this offset in it's
//Library and in rendering code it's used to "Center" the tiles onto the bevy map
impl MapCoordinates {
    fn units_per_tile(&self) -> Vec2 {
        self.tile_size.into()
    }

    fn bottom_left_offset(&self, map_size_in_tiles: UVec2) -> Vec2 {
        self.units_per_tile() * map_size_in_tiles.as_vec2() * -0.5
    }

    pub fn world_to_tile_pos(&self, map_size_in_tiles: UVec2, world_pos: Vec2) -> Option<UVec2> {
        // Center the world position based on the map size and origin
        let map_position_in_units =
            world_pos - self.world_origin - self.bottom_left_offset(map_size_in_tiles);

        // Convert world position to tile coordinates by dividing by the size of each tile
        let tile_index = map_position_in_units / self.units_per_tile();

        // Check if tile is positive and within map bounds, then convert to UVec2 if valid
        (tile_index.x >= 0.0 && tile_index.y >= 0.0)
            .then(|| tile_index.as_uvec2())
            .filter(|tile_pos| tile_pos.x < map_size_in_tiles.x && tile_pos.y < map_size_in_tiles.y)
    }

    pub fn tile_space_to_world_pos(&self, map_size_in_tiles: UVec2, tile_pos: Vec2) -> Vec2 {
        // Convert tile position to world position by multiplying by the size of each tile
        let world_pos = tile_pos * self.units_per_tile();

        let map_origin = self.world_origin + self.bottom_left_offset(map_size_in_tiles);

        // Center the world position based on the map size and origin
        world_pos + map_origin
    }

    // example: if tile_index is (2,3) and units_per_tile is (32,32), then tile_center is (80,112) (2.5 * 32, 3.5 * 32)
    pub fn tile_center_to_world_pos(&self, map_size_in_tiles: UVec2, tile_index: IVec2) -> Vec2 {
        self.tile_space_to_world_pos(map_size_in_tiles, tile_index.as_vec2() + Vec2::splat(0.5))
    }

    pub fn tile_corner_to_world_pos(&self, map_size_in_tiles: UVec2, tile_index: IVec2) -> Vec2 {
        self.tile_space_to_world_pos(map_size_in_tiles, tile_index.as_vec2())
    }

    pub fn tile_rect_to_world_rect(
        &self,
        map_size_in_tiles: UVec2,
        tile_rect: TileRect,
    ) -> (Vec2, Vec2) {
        let rect_size_in_units = tile_rect.size.as_vec2() * self.units_per_tile();

        // bottom-left corner of the rect in world space
        let min_world = self.tile_corner_to_world_pos(map_size_in_tiles, tile_rect.min);

        // true geometric center
        let center = min_world + rect_size_in_units / 2.0;

        (center, rect_size_in_units)
    }
}
