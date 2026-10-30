//! Picks tile atlas appearances from tile kinds, a visual seed, and neighbours.
//!
//! Each [`TileKind`] maps to one [`TilingRule`]: plain kinds roll a
//! deterministic centre variant, tinted kinds add a seeded brightness, walls
//! pick a top or face piece from wall neighbours, and banks pick transition
//! pieces where they meet land. Pure grid math with no backend detail;
//! spawning stays in `spawn.rs`.

use bevy::prelude::IVec2;

use super::tiles::{TileGrid, TileKind};

/// Geometry of `tile_atlas.png`: first flat index of each art block.
struct AtlasLayout {
    /// First index of the 10-tile centre-variant rows.
    ground_first: u32,
    grass_first: u32,
    cobble_first: u32,
    water_first: u32,
    wood_first: u32,
    /// First index of the one-sided bank rows.
    water_bank_first: u32,
    grass_bank_first: u32,
    /// Wall caps for vertical runs, keyed by flank mask (bit0 exposed west,
    /// bit1 exposed east): blank, west cap, east cap, both caps.
    wall_top_mid: [u32; 4],
    /// Wall caps for run tops, same flank mask: top, top-left, top-right, all.
    wall_top_edge: [u32; 4],
    /// Wall face pieces keyed by side mask (bit0 wall west, bit1 wall east):
    /// isolated, east end, west end. Mask 3 (both sides) rolls a straight
    /// variant instead. Only the outlined-base pieces show: a face implies
    /// no wall below.
    wall_face_ends: [u32; 3],
    /// Straight face variants with an outlined bottom row.
    wall_face_straights: [u32; 3],
    /// Bank piece offset keyed by land mask (bit0 N, bit1 S, bit2 W, bit3 E);
    /// `None` rolls a centre variant instead.
    bank_offsets: [Option<u32>; 16],
}

/// Layout of the atlas: centre rows with blank separator rows between types
/// (ground, grass, cobble, water, wood), wall pieces in row 10 (overflow
/// 110-119), and bank rows holding edge, corner, and channel pieces with
/// three-sided pocket pieces (rows 16/18) below.
const ATLAS: AtlasLayout = AtlasLayout {
    ground_first: 0,
    grass_first: 20,
    cobble_first: 40,
    water_first: 60,
    wood_first: 80,
    water_bank_first: 120,
    grass_bank_first: 140,
    wall_top_mid: [113, 111, 110, 112],
    wall_top_edge: [100, 101, 102, 103],
    wall_face_ends: [119, 118, 117],
    wall_face_straights: [114, 115, 116],
    bank_offsets: [
        None,     // 0: open centre
        Some(0),  // N edge
        Some(1),  // S edge
        Some(8),  // N+S channel
        Some(2),  // W edge
        Some(5),  // N+W corner
        Some(7),  // S+W corner
        Some(40), // N+S+W pocket
        Some(3),  // E edge
        Some(4),  // N+E corner
        Some(6),  // S+E corner
        Some(41), // N+S+E pocket
        Some(9),  // W+E channel
        Some(43), // N+W+E pocket
        Some(42), // S+W+E pocket
        Some(8),  // surrounded stripe, never a hard full tile
    ],
};

/// Seeded brightness multipliers for tinted tiles (cobblestone).
const TINT_BRIGHTNESS: [f32; 3] = [1.0, 0.98, 0.96];

/// How a tile kind maps to atlas art.
#[derive(Debug, Clone, Copy)]
enum TilingRule {
    /// Fixed atlas slot with a seeded brightness from `brightness`.
    Tinted {
        index: u32,
        brightness: &'static [f32],
    },
    /// One of `count` centre variants starting at `first`.
    Centres { first: u32, count: u16 },
    /// Side-view wall pieces picked from wall neighbours.
    Wall,
    /// Centre variants, switching to bank pieces where land meets.
    Bank { centre_first: u32, bank_first: u32 },
}

/// Returns the tiling rule for a kind, or `None` when it should not render.
///
/// **If a kind renders as the wrong art, this function is the first place to
/// check.** Adding a kind is a new row here, not a new code path.
fn tiling_rule(tile_kind: TileKind) -> Option<TilingRule> {
    match tile_kind {
        // Cobblestone is one tileable tile, so grooves line up between
        // neighbours by construction; a seeded tint is the only per-tile
        // variation. (Flips would break the line-up, so cobble never flips.)
        TileKind::Cobblestone => Some(TilingRule::Tinted {
            index: ATLAS.cobble_first,
            brightness: &TINT_BRIGHTNESS,
        }),
        TileKind::Ground => Some(TilingRule::Centres {
            first: ATLAS.ground_first,
            count: 10,
        }),
        TileKind::Wood => Some(TilingRule::Centres {
            first: ATLAS.wood_first,
            count: 10,
        }),
        TileKind::Grass => Some(TilingRule::Bank {
            centre_first: ATLAS.grass_first,
            bank_first: ATLAS.grass_bank_first,
        }),
        TileKind::Water => Some(TilingRule::Bank {
            centre_first: ATLAS.water_first,
            bank_first: ATLAS.water_bank_first,
        }),
        TileKind::Wall => Some(TilingRule::Wall),
        TileKind::DeadZone => None,
    }
}

/// Atlas slot and tint for one rendered tile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TileAppearance {
    /// Flat atlas index for `TileTextureIndex`.
    pub texture_index: u32,
    /// Grayscale brightness multiplier, or `None` for no tint.
    pub brightness: Option<f32>,
}

/// Maps a tile to its atlas appearance, or `None` when it should not render.
pub(crate) fn tile_appearance(
    tiles: &TileGrid<TileKind>,
    seed: u64,
    tile: IVec2,
    tile_kind: TileKind,
) -> Option<TileAppearance> {
    let plain = |texture_index: u32| TileAppearance {
        texture_index,
        brightness: None,
    };
    Some(match tiling_rule(tile_kind)? {
        TilingRule::Tinted { index, brightness } => {
            let shade = tile_variant(seed, tile_kind, tile, brightness.len() as u16) as usize;
            TileAppearance {
                texture_index: index,
                brightness: Some(brightness[shade]),
            }
        }
        TilingRule::Centres { first, count } => {
            plain(first + u32::from(tile_variant(seed, tile_kind, tile, count)))
        }
        TilingRule::Wall => plain(wall_texture_index(&ATLAS, tiles, seed, tile)),
        TilingRule::Bank {
            centre_first,
            bank_first,
        } => plain(bank_texture_index(
            &ATLAS,
            tiles,
            seed,
            tile,
            tile_kind,
            centre_first,
            bank_first,
        )),
    })
}

/// How off-map neighbours read when sampling a tile's surroundings.
#[derive(Debug, Clone, Copy)]
enum EdgePolicy {
    /// Off-map tiles read as `TileKind::Wall`, closing the map border ring.
    Closed,
    /// Off-map tiles read as the sampled kind, drawing no edge transition.
    Open,
}

impl EdgePolicy {
    /// Returns the kind an off-map tile reads as.
    fn fallback(self, tile_kind: TileKind) -> TileKind {
        match self {
            Self::Closed => TileKind::Wall,
            Self::Open => tile_kind,
        }
    }
}

/// Returns the kind at an offset, treating off-map tiles per `edge`.
fn neighbour_kind(
    tiles: &TileGrid<TileKind>,
    tile: IVec2,
    offset: IVec2,
    edge: EdgePolicy,
    tile_kind: TileKind,
) -> TileKind {
    tiles.get(tile + offset).unwrap_or(edge.fallback(tile_kind))
}

/// Picks a side-view wall piece from wall neighbours (`+y` is north).
///
/// A wall with more mass to the south shows its cap; north-south runs show only
/// their caps (the camera never sees their faces); otherwise the brick face
/// with a cap lip shows. Caps border only exposed edges: run tops always lip
/// the north side, flanks only where no wall neighbour sits.
fn wall_texture_index(
    layout: &AtlasLayout,
    tiles: &TileGrid<TileKind>,
    seed: u64,
    tile: IVec2,
) -> u32 {
    let wall = |offset: IVec2| {
        neighbour_kind(tiles, tile, offset, EdgePolicy::Closed, TileKind::Wall) == TileKind::Wall
    };
    let north_wall = wall(IVec2::new(0, 1));
    let south_wall = wall(IVec2::new(0, -1));
    let east_wall = wall(IVec2::new(1, 0));
    let west_wall = wall(IVec2::new(-1, 0));
    // Flank mask: bit0 exposed west, bit1 exposed east.
    let flank = u8::from(!west_wall) | (u8::from(!east_wall) << 1);
    if north_wall && south_wall {
        // Cap continues vertically: border only the exposed flanks.
        layout.wall_top_mid[flank as usize]
    } else if south_wall {
        // Run top (no wall north): the north lip always shows; flanks only
        // when exposed.
        layout.wall_top_edge[flank as usize]
    } else {
        // Only outlined-base pieces show: a face implies no wall below.
        let side = u8::from(west_wall) | (u8::from(east_wall) << 1);
        if side == 3 {
            let pick = tile_variant(
                seed,
                TileKind::Wall,
                tile,
                layout.wall_face_straights.len() as u16,
            ) as usize;
            layout.wall_face_straights[pick]
        } else {
            layout.wall_face_ends[side as usize]
        }
    }
}

/// Picks a centre variant or a one-sided bank piece where `tile_kind` meets land.
///
/// Water rims grass directly with no dirt in between; grass breaks only onto
/// ground, staying plain beside water and walls.
fn bank_texture_index(
    layout: &AtlasLayout,
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
    let land = |offset: IVec2| {
        is_land(neighbour_kind(
            tiles,
            tile,
            offset,
            EdgePolicy::Open,
            tile_kind,
        ))
    };
    // Land mask: bit0 N, bit1 S, bit2 W, bit3 E.
    let mask = u8::from(land(IVec2::new(0, 1)))
        | (u8::from(land(IVec2::new(0, -1))) << 1)
        | (u8::from(land(IVec2::new(-1, 0))) << 2)
        | (u8::from(land(IVec2::new(1, 0))) << 3);
    match layout.bank_offsets[mask as usize] {
        Some(offset) => bank_first + offset,
        None => centre_first + u32::from(tile_variant(seed, tile_kind, tile, 10)),
    }
}

/// Returns a deterministic variant in `0..variant_count` for a tile.
///
/// Identical `(seed, kind, tile)` triples always map to the same variant;
/// adjacent tiles get decorrelated values. The splitmix64 finalizer is what
/// spreads the low bits of `tile.x`/`tile.y` so neighbouring tiles don't all
/// pick the same variant. Salted by the kind discriminant, so re-encoding a
/// map glyph never reshuffles art.
fn tile_variant(seed: u64, tile_kind: TileKind, tile: IVec2, variant_count: u16) -> u16 {
    debug_assert!(
        variant_count > 0,
        "the hash is reduced modulo variant_count, which divides by zero"
    );

    let mut hash = seed
        ^ u64::from(tile_kind as u32).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ u64::from(tile.x as u32).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ u64::from(tile.y as u32).wrapping_mul(0x1656_67B1_9E37_79F9);

    hash ^= hash >> 30;
    hash = hash.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    hash ^= hash >> 27;
    hash = hash.wrapping_mul(0x94D0_49BB_1331_11EB);
    hash ^= hash >> 31;

    (hash % u64::from(variant_count)) as u16
}
