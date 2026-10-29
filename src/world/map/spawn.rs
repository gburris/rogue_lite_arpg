use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::BTreeMap;

use crate::{
    prelude::*,
    world::map::blueprint::{MapBlueprint, PlacementKind},
    world::map::tiling::tile_appearance,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin);
    app.init_resource::<TileToWorld>();
    app.add_systems(
        OnEnter(AppState::SpawnMap),
        (
            set_tile_to_world_map_size,
            populate_map_tiles,
            spawn_colliders,
            spawn_placements,
            transition_to_playing,
        )
            .chain(),
    );
}

fn transition_to_playing(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::Playing);
}

fn set_tile_to_world_map_size(
    blueprint: Single<&MapBlueprint>,
    mut coordinates: ResMut<TileToWorld>,
) {
    coordinates.map_size = blueprint.map_size();
}

fn populate_map_tiles(
    mut commands: Commands,
    blueprint: Single<(&MapBlueprint, Entity)>,
    coordinates: Res<TileToWorld>,
    tiles: Res<TileAssets>,
) {
    let (blueprint, map) = blueprint.into_inner();
    spawn_tiles(&mut commands, map, blueprint, &tiles, &coordinates);
}

fn spawn_tiles(
    commands: &mut Commands,
    map: Entity,
    blueprint: &MapBlueprint,
    tiles: &TileAssets,
    coordinates: &TileToWorld,
) {
    let map_size = blueprint.map_size();
    let grid_size: TilemapGridSize = coordinates.tile_size.into();
    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size.into());

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_index = IVec2::new(x as i32, y as i32);
            let Some(tile_kind) = blueprint.tiles.get(tile_index) else {
                continue;
            };
            let Some(appearance) =
                tile_appearance(&blueprint.tiles, blueprint.seed, tile_index, tile_kind)
            else {
                continue;
            };

            let tile_position = TilePos { x, y };
            // Seeded tint comes from tiling; nothing else varies per tile.
            let color = appearance
                .brightness
                .map_or_else(TileColor::default, |brightness| {
                    TileColor(Color::srgb(brightness, brightness, brightness))
                });
            let tile_entity = commands
                .spawn((
                    Name::new("Tile"),
                    TileBundle {
                        position: tile_position,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(appearance.texture_index),
                        color,
                        ..default()
                    },
                    ChildOf(tilemap_entity),
                ))
                .id();
            storage.set(&tile_position, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert((
        Name::new("Tiles"),
        TilemapBundle {
            grid_size,
            size: map_size.into(),
            storage,
            map_type: TilemapType::Square,
            texture: TilemapTexture::Single(tiles.atlas.clone()),
            tile_size: coordinates.tile_size,
            anchor: TilemapAnchor::Center,
            transform: Transform::default(),
            ..default()
        },
        ChildOf(map),
        // The one exception to "only NextMap::set despawns map pieces": game over clears
        // the tiles so its screen shows no half-empty map. The map entity itself stays
        // until the next NextMap::set.
        DespawnOnExit::<AppState>(AppState::Playing),
    ));
}

// ---------------------------------------------------------------------------
// Colliders
// ---------------------------------------------------------------------------

/// Collider marker for map geometry that is removed when gameplay ends.
#[derive(Component)]
#[require(DespawnOnExit::<AppState>(AppState::Playing))]
pub struct MapCollider;

fn spawn_colliders(
    mut commands: Commands,
    blueprint: Single<&MapBlueprint>,
    coordinates: Res<TileToWorld>,
) {
    for rect in derive_collider_rects(&blueprint.tiles) {
        let (position, size) = coordinates.tile_rectangle_to_world_rectangle(rect);

        commands.spawn((
            RigidBody::Static,
            Collider::rectangle(size.x, size.y),
            Transform::from_xyz(position.x, position.y, ZLayer::OnGround.z()),
            MapCollider,
            CollisionLayers::new(
                GameCollisionLayer::HighObstacle,
                GameCollisionLayer::HIGH_OBSTACLE_FILTERS,
            ),
        ));
    }
}

fn derive_collider_rects(tiles: &TileGrid<TileKind>) -> Vec<TileRect> {
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

        completed_collider_rects.extend(previous_runs.into_values());
        previous_runs = continued_runs;
    }

    completed_collider_rects.extend(previous_runs.into_values());
    completed_collider_rects
}

fn blocking_runs(tiles: &TileGrid<TileKind>, y: i32, map_width: i32) -> Vec<(i32, i32)> {
    let mut runs = Vec::new();
    let mut x = 0;

    while x < map_width {
        while x < map_width
            && !tiles
                .get(IVec2::new(x, y))
                .is_some_and(|tile_kind| tile_kind.blocks_ground())
        {
            x += 1;
        }

        let start = x;

        while x < map_width
            && tiles
                .get(IVec2::new(x, y))
                .is_some_and(|tile_kind| tile_kind.blocks_ground())
        {
            x += 1;
        }

        if start < x {
            runs.push((start, x));
        }
    }

    runs
}

// ---------------------------------------------------------------------------
// Placements
// ---------------------------------------------------------------------------

fn spawn_placements(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    blueprint: Single<&MapBlueprint>,
    coordinates: Res<TileToWorld>,
    mut player: Query<&mut Transform, With<Player>>,
) -> Result {
    let mut player_transform = player.single_mut()?;
    let mut enemies = Vec::new();
    let mut npcs = Vec::new();
    let mut chests = Vec::new();

    for placement in &blueprint.placements {
        let world_spawn_position = coordinates.tile_center_to_world_position(placement.tile);

        match &placement.kind {
            PlacementKind::PlayerStart => {
                player_transform.translation =
                    world_spawn_position.extend(player_transform.translation.z);
            }
            PlacementKind::Enemy(enemy_kind) => {
                enemies.push(EnemySpawnData {
                    position: world_spawn_position,
                    enemy_kind: *enemy_kind,
                });
            }
            PlacementKind::Npc(npc_kind) => {
                npcs.push(NpcSpawnData {
                    position: world_spawn_position,
                    npc_kind: *npc_kind,
                });
            }
            PlacementKind::Chest => chests.push(world_spawn_position),
            PlacementKind::Exit => {
                commands.spawn(portal(&sprites, world_spawn_position));
            }
        }
    }

    if !enemies.is_empty() {
        commands.trigger(SpawnEnemies(enemies));
    }
    if !npcs.is_empty() {
        commands.trigger(SpawnNpcs(npcs));
    }
    if !chests.is_empty() {
        commands.trigger(SpawnChestsEvent(chests));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tile <-> world conversion
// ---------------------------------------------------------------------------

/// Largest map width or height, in tiles, that recipe validation accepts.
pub const MAX_MAP_TILES: u32 = 256;
/// Width and height of a square map tile, in world units.
pub const TILE_SIZE: u32 = 32;

/// Converts tile coordinates and dimensions to world-space positions.
#[derive(Resource, Clone, Copy)]
pub struct TileToWorld {
    /// Width and height of one tile, in world units.
    pub tile_size: TilemapTileSize,
    /// Map width and height, in tiles.
    pub map_size: UVec2,
}

impl Default for TileToWorld {
    fn default() -> Self {
        Self {
            tile_size: TilemapTileSize::new(TILE_SIZE as f32, TILE_SIZE as f32),
            map_size: UVec2::ZERO,
        }
    }
}

impl TileToWorld {
    fn units_per_tile(&self) -> Vec2 {
        self.tile_size.into()
    }

    fn bottom_left_offset(&self) -> Vec2 {
        self.units_per_tile() * self.map_size.as_vec2() * -0.5
    }

    /// Returns the tile containing a world-space position, or `None` outside the map.
    pub fn world_to_tile_position(&self, world_position: Vec2) -> Option<IVec2> {
        let map_position_in_units = world_position - self.bottom_left_offset();
        let tile_index = (map_position_in_units / self.units_per_tile())
            .floor()
            .as_ivec2();

        (tile_index.cmpge(IVec2::ZERO).all() && tile_index.cmplt(self.map_size.as_ivec2()).all())
            .then_some(tile_index)
    }

    /// Converts tile-space coordinates to world-space coordinates.
    pub fn tile_space_to_world_position(&self, tile_coordinates: Vec2) -> Vec2 {
        tile_coordinates * self.units_per_tile() + self.bottom_left_offset()
    }

    /// Converts a tile index to the center of that tile in world space.
    pub fn tile_center_to_world_position(&self, tile_index: IVec2) -> Vec2 {
        self.tile_space_to_world_position(tile_index.as_vec2() + Vec2::splat(0.5))
    }

    /// Converts a tile index to its lower-left corner in world space.
    pub fn tile_corner_to_world_position(&self, tile_index: IVec2) -> Vec2 {
        self.tile_space_to_world_position(tile_index.as_vec2())
    }

    /// Returns a tile rectangle's world-space center and dimensions.
    pub fn tile_rectangle_to_world_rectangle(&self, tile_rectangle: TileRect) -> (Vec2, Vec2) {
        let rectangle_size_in_units = tile_rectangle.size.as_vec2() * self.units_per_tile();
        let rectangle_minimum_world_position =
            self.tile_corner_to_world_position(tile_rectangle.min);
        let rectangle_center = rectangle_minimum_world_position + rectangle_size_in_units / 2.0;
        (rectangle_center, rectangle_size_in_units)
    }
}
