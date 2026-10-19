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
            let Some(texture_index) = tile_texture_index(
                &blueprint.tiles,
                blueprint.visual_seed,
                tile_index,
                tile_kind,
            ) else {
                continue;
            };

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

/// First flat index of each 10-tile art row (blank separator rows between types):
/// ground, grass, cobble, water, wood.
const CENTRE_ROW_FIRST: [u32; 5] = [0, 20, 40, 60, 80];
/// Wall pieces (row 10): 4 top caps, 3 straight faces, ends, singles.
const WALL_TOP_STRAIGHT: u32 = 100;
const WALL_TOP_END_WEST: u32 = 101;
const WALL_TOP_END_EAST: u32 = 102;
const WALL_TOP_SINGLE: u32 = 103;
const WALL_FACE_STRAIGHTS: [u32; 3] = [104, 105, 109];
const WALL_FACE_END_WEST: u32 = 106;
const WALL_FACE_END_EAST: u32 = 107;
const WALL_FACE_SINGLE: u32 = 108;
/// One-sided bank pieces (water row 12, grass-dirt row 14):
/// N/S/W/E edge, NE/NW/SE/SW convex corners, horizontal/vertical channels.
const WATER_BANK_FIRST: u32 = 120;
const GRASS_DIRT_FIRST: u32 = 140;

/// Maps a tile to its atlas texture index, or `None` when it should not render.
///
/// **If a kind renders as the wrong art, this function is the first place to
/// check.** Plain kinds roll a deterministic variant; walls pick a top/face
/// piece from wall neighbours; water and grass pick bank pieces where they
/// meet land.
fn tile_texture_index(
    tiles: &TileGrid<TileKind>,
    seed: u64,
    tile: IVec2,
    tile_kind: TileKind,
) -> Option<u32> {
    match tile_kind {
        TileKind::Ground | TileKind::Cobblestone | TileKind::Wood => {
            let first = match tile_kind {
                TileKind::Ground => CENTRE_ROW_FIRST[0],
                TileKind::Cobblestone => CENTRE_ROW_FIRST[2],
                _ => CENTRE_ROW_FIRST[4],
            };
            Some(first + u32::from(tile_variant(seed, tile_kind.glyph(), tile, 10)))
        }
        TileKind::Grass => Some(bank_texture_index(
            tiles,
            seed,
            tile,
            TileKind::Grass,
            CENTRE_ROW_FIRST[1],
            GRASS_DIRT_FIRST,
        )),
        TileKind::Water => Some(bank_texture_index(
            tiles,
            seed,
            tile,
            TileKind::Water,
            CENTRE_ROW_FIRST[3],
            WATER_BANK_FIRST,
        )),
        TileKind::Wall => Some(wall_texture_index(tiles, seed, tile)),
        TileKind::DeadZone => None,
    }
}

/// Returns the kind at an offset, treating off-map tiles as `fallback`.
///
/// Walls fall back to `Wall` so the map border ring renders closed ends;
/// banks fall back to their own kind so no transition draws at the map edge.
fn neighbour_kind(
    tiles: &TileGrid<TileKind>,
    tile: IVec2,
    offset_x: i32,
    offset_y: i32,
    fallback: TileKind,
) -> TileKind {
    tiles
        .get(tile + IVec2::new(offset_x, offset_y))
        .unwrap_or(fallback)
}

/// Picks a side-view wall piece from wall neighbours (`+y` is north).
///
/// A wall with more mass to the south shows its cap; north-south runs show only
/// their caps (the camera never sees their faces); otherwise the brick face
/// with a cap lip shows. East/west neighbours pick runs, ends, or singles.
fn wall_texture_index(tiles: &TileGrid<TileKind>, seed: u64, tile: IVec2) -> u32 {
    let north_wall = neighbour_kind(tiles, tile, 0, 1, TileKind::Wall) == TileKind::Wall;
    let south_wall = neighbour_kind(tiles, tile, 0, -1, TileKind::Wall) == TileKind::Wall;
    let east_wall = neighbour_kind(tiles, tile, 1, 0, TileKind::Wall) == TileKind::Wall;
    let west_wall = neighbour_kind(tiles, tile, -1, 0, TileKind::Wall) == TileKind::Wall;
    if north_wall && south_wall && east_wall && west_wall {
        // Deep interior sees no open side: unbroken cap, no lip striping.
        return WALL_TOP_STRAIGHT;
    }
    if south_wall && !north_wall {
        match (west_wall, east_wall) {
            (true, true) => WALL_TOP_STRAIGHT,
            (false, true) => WALL_TOP_END_WEST,
            (true, false) => WALL_TOP_END_EAST,
            (false, false) => WALL_TOP_SINGLE,
        }
    } else if north_wall && south_wall && !(east_wall && west_wall) {
        WALL_TOP_STRAIGHT
    } else {
        match (west_wall, east_wall) {
            (true, true) => {
                let pick = tile_variant(seed, TileKind::Wall.glyph(), tile, 3);
                WALL_FACE_STRAIGHTS[pick as usize]
            }
            (false, true) => WALL_FACE_END_WEST,
            (true, false) => WALL_FACE_END_EAST,
            (false, false) => WALL_FACE_SINGLE,
        }
    }
}

/// Picks a centre variant or a one-sided bank piece where `tile_kind` meets land.
///
/// Water rims grass directly with no dirt in between; grass breaks only onto
/// ground, staying plain beside water and walls.
fn bank_texture_index(
    tiles: &TileGrid<TileKind>,
    seed: u64,
    tile: IVec2,
    tile_kind: TileKind,
    centre_first: u32,
    bank_first: u32,
) -> u32 {
    let is_land = |kind: TileKind| match tile_kind {
        TileKind::Water => kind != TileKind::Water && kind != TileKind::DeadZone,
        _ => kind == TileKind::Ground,
    };
    let north_land = is_land(neighbour_kind(tiles, tile, 0, 1, tile_kind));
    let south_land = is_land(neighbour_kind(tiles, tile, 0, -1, tile_kind));
    let west_land = is_land(neighbour_kind(tiles, tile, -1, 0, tile_kind));
    let east_land = is_land(neighbour_kind(tiles, tile, 1, 0, tile_kind));
    let centre = || centre_first + u32::from(tile_variant(seed, tile_kind.glyph(), tile, 10));
    match (north_land, south_land, west_land, east_land) {
        (false, false, false, false) => centre(),
        (true, false, false, false) => bank_first,
        (false, true, false, false) => bank_first + 1,
        (false, false, true, false) => bank_first + 2,
        (false, false, false, true) => bank_first + 3,
        (true, false, false, true) => bank_first + 4,
        (true, false, true, false) => bank_first + 5,
        (false, true, false, true) => bank_first + 6,
        (false, true, true, false) => bank_first + 7,
        // Narrow runs reuse the channel along their open axis.
        (true, true, false, false) => bank_first + 8,
        (false, false, true, true) => bank_first + 9,
        // Three-sided dead ends get pocket pieces (rows 16/18) open to water.
        (true, true, true, false) => bank_first + 40,
        (true, true, false, true) => bank_first + 41,
        (false, true, true, true) => bank_first + 42,
        (true, false, true, true) => bank_first + 43,
        // Fully surrounded: keep the middle stripe, never a hard full tile.
        (true, true, true, true) => bank_first + 8,
        _ => centre(),
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
