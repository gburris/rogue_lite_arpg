use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::BTreeMap;

use crate::{
    prelude::*,
    world::map::blueprint::{MapBlueprint, PlacementKind},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin);
    app.init_resource::<TileToWorld>();
    app.add_systems(
        OnEnter(AppState::SpawnMap),
        (
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

fn populate_map_tiles(
    mut commands: Commands,
    blueprint: Single<(&MapBlueprint, Entity)>,
    mut coordinates: ResMut<TileToWorld>,
    tiles: Res<TileAssets>,
) {
    let (blueprint, map) = blueprint.into_inner();
    coordinates.map_size = blueprint.map_size();
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
            let Some(slot) = atlas_slot(tile_kind) else {
                continue;
            };
            let variant = tile_variant(
                blueprint.visual_seed,
                tile_kind.glyph(),
                tile_index,
                slot.variant_count,
            );
            let texture_index = slot.first_variant + variant;

            let tile_position = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    Name::new("Tile"),
                    TileBundle {
                        position: tile_position,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(u32::from(texture_index)),
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
        DespawnOnExit::<AppState>(AppState::Playing),
    ));
}

// ---------------------------------------------------------------------------
// Tile atlas mapping
// ---------------------------------------------------------------------------

/// Where one tile kind's art lives in the tile atlas.
///
/// The atlas is indexed row-major as a flat `u32`. Each kind owns a contiguous
/// block of `variant_count` texture indices starting at `first_variant`.
#[derive(Clone, Copy, Debug)]
struct AtlasSlot {
    first_variant: u16,
    variant_count: u16,
}

impl AtlasSlot {
    const fn new(first_variant: u16, variant_count: u16) -> Self {
        Self {
            first_variant,
            variant_count,
        }
    }
}

/// Maps a logical tile kind to its block of atlas variants.
///
/// **If a kind renders as the wrong art, this table is the first place to
/// check.** The atlas is a single vertical column of 60 tiles — 10 variants
/// stacked per kind — so the flat texture index equals the row index:
fn atlas_slot(tile_kind: TileKind) -> Option<AtlasSlot> {
    match tile_kind {
        TileKind::Ground => Some(AtlasSlot::new(0, 10)),
        TileKind::Grass => Some(AtlasSlot::new(10, 10)),
        TileKind::Cobblestone => Some(AtlasSlot::new(20, 10)),
        TileKind::Water => Some(AtlasSlot::new(30, 10)),
        TileKind::Wood => Some(AtlasSlot::new(40, 10)),
        TileKind::Wall => Some(AtlasSlot::new(50, 10)),
        TileKind::DeadZone => None,
    }
}
/// Returns a deterministic variant in `0..variant_count` for a tile.
///
/// Identical `(seed, glyph, tile)` triples always map to the same variant;
/// adjacent tiles get decorrelated values. The splitmix64 finalizer is what
/// spreads the low bits of `tile.x`/`tile.y` so neighbouring tiles don't all
/// pick the same variant.
fn tile_variant(seed: u64, glyph: char, tile: IVec2, variant_count: u16) -> u16 {
    debug_assert!(variant_count > 0, "variant_count must be non-zero");

    let mut hash = seed
        ^ u64::from(glyph as u32).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ u64::from(tile.x as u32).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ u64::from(tile.y as u32).wrapping_mul(0x1656_67B1_9E37_79F9);

    hash ^= hash >> 30;
    hash = hash.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    hash ^= hash >> 27;
    hash = hash.wrapping_mul(0x94D0_49BB_1331_11EB);
    hash ^= hash >> 31;

    (hash % u64::from(variant_count)) as u16
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
    player: Option<Single<&mut Transform, With<Player>>>,
) {
    let mut player_transform = player.map(|player| player.into_inner());
    let mut enemies = Vec::new();
    let mut npcs = Vec::new();
    let mut chests = Vec::new();

    for placement in &blueprint.placements {
        let world_spawn_position = coordinates.tile_center_to_world_position(placement.tile);

        match &placement.kind {
            PlacementKind::PlayerStart => {
                let Some(transform) = player_transform.as_deref_mut() else {
                    warn!("PlayerStart placement found but no Player entity exists");
                    continue;
                };
                transform.translation = world_spawn_position.extend(transform.translation.z);
            }
            PlacementKind::Enemy(enemy_kind) => {
                enemies.push(EnemySpawnData {
                    position: world_spawn_position,
                    enemy_kind: enemy_kind.clone(),
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
}

// ---------------------------------------------------------------------------
// Tile <-> world conversion
// ---------------------------------------------------------------------------

/// Normalization limit for map-depth sorting; not a gameplay size cap.
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
    pub fn world_to_tile_position(&self, world_position: Vec2) -> Option<UVec2> {
        let map_position_in_units = world_position - self.bottom_left_offset();
        let tile_index = map_position_in_units / self.units_per_tile();

        (tile_index.x >= 0.0 && tile_index.y >= 0.0)
            .then(|| tile_index.as_uvec2())
            .filter(|tile_position| {
                tile_position.x < self.map_size.x && tile_position.y < self.map_size.y
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilemap_backend_agrees_with_tile_to_world_at_7_by_4_endpoints() {
        let map_size = UVec2::new(7, 4);
        let coordinates = TileToWorld {
            map_size,
            ..default()
        };
        let tilemap_size: TilemapSize = map_size.into();
        let grid_size: TilemapGridSize = coordinates.tile_size.into();
        let tilemap_type = TilemapType::Square;
        let anchor = TilemapAnchor::Center;
        for tile_position in [TilePos::new(0, 0), TilePos::new(6, 3)] {
            let backend_center = tile_position.center_in_world(
                &tilemap_size,
                &grid_size,
                &coordinates.tile_size,
                &tilemap_type,
                &anchor,
            );
            assert_eq!(
                backend_center,
                coordinates.tile_center_to_world_position(IVec2::new(
                    tile_position.x as i32,
                    tile_position.y as i32,
                ))
            );
        }
    }
}
