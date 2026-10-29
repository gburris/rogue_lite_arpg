use bevy::prelude::{IVec2, UVec2, Vec2};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;

use crate::world::map::blueprint::{MapBlueprint, MapError, Placement, PlacementKind};
use crate::world::map::placement::{PlaceContext, PlaceRule, place};
use crate::world::map::tiles::{TileGrid, TileKind, TileRect};

/// Tile data and placement markers for a reusable map structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    /// Tile values, with `None` marking cells left unchanged when applied.
    pub tiles: TileGrid<Option<TileKind>>,
    /// Placement markers stored in coordinates local to this structure.
    pub placements: Vec<Placement>,
}

/// Parses top-to-bottom glyph rows into a bottom-left-origin tile and placement grid.
/// `:` leaves the destination tile untouched when the structure is applied.
pub fn structure_from_glyphs(floor: TileKind, text: &str) -> Result<Structure, MapError> {
    let rows: Vec<_> = text.trim_matches(['\r', '\n']).lines().collect();
    let Some(first_row) = rows.first() else {
        return Err("structure glyphs are empty".into());
    };
    let width = first_row.chars().count();
    if let Some((row_index, _)) = rows
        .iter()
        .enumerate()
        .find(|(_, row_text)| row_text.chars().count() != width)
    {
        return Err(format!("structure glyph row {row_index} has uneven width").into());
    }
    let (Ok(grid_width), Ok(grid_height)) = (u32::try_from(width), u32::try_from(rows.len()))
    else {
        return Err("structure glyph dimensions exceed the supported range".into());
    };
    if grid_width == 0 || grid_height == 0 {
        return Err("structure glyphs are empty".into());
    }
    let mut tiles = TileGrid::new(UVec2::new(grid_width, grid_height), None);
    let mut placements = Vec::new();
    for (row_index, row_text) in rows.iter().enumerate() {
        for (column_index, glyph) in row_text.chars().enumerate() {
            let local_tile_position = IVec2::new(
                column_index as i32,
                grid_height as i32 - 1 - row_index as i32,
            );
            let tile_kind = TileKind::from_glyph(glyph);
            let placement_kind = PlacementKind::from_glyph(glyph);
            if glyph != ':' && tile_kind.is_none() && placement_kind.is_none() {
                return Err(format!(
                    "unknown structure glyph {glyph:?} at row {row_index}, column {column_index}"
                )
                .into());
            }
            tiles.set(
                local_tile_position,
                tile_kind.or((glyph != ':').then_some(floor)),
            );
            if let Some(kind) = placement_kind {
                placements.push(Placement {
                    tile: local_tile_position,
                    kind,
                });
            }
        }
    }
    Ok(Structure { tiles, placements })
}

/// Applies non-empty cells and placement markers at the supplied bottom-left origin.
pub fn apply_structure(
    blueprint: &mut MapBlueprint,
    structure: &Structure,
    structure_origin: IVec2,
) -> Result<(), MapError> {
    let structure_rect = TileRect {
        min: structure_origin,
        size: structure.tiles.map_size(),
    };
    if !blueprint.tiles.contains_rect(&structure_rect) {
        return Err("structure placement is out of bounds".into());
    }
    for (local_position, tile_kind) in structure.tiles.iter() {
        if let Some(tile_kind) = tile_kind {
            blueprint
                .tiles
                .set(structure_origin + local_position, tile_kind);
        }
    }
    blueprint
        .placements
        .extend(structure.placements.iter().map(|placement| Placement {
            tile: structure_origin + placement.tile,
            kind: placement.kind.clone(),
        }));
    Ok(())
}

/// Footprint size for a builtin feature: an exact rectangle or an
/// independently rolled rectangle between minimum and maximum corners.
/// RON spells `size: Exact((14, 9))` or `size: Ranged((6, 4), (12, 8))`;
/// omitting it keeps the feature's classic square size roll.
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum FeatureSize {
    /// Fixed `(width, height)` footprint in tiles.
    Exact((u32, u32)),
    /// Independently rolled `(width, height)` between the two corners.
    Ranged((u32, u32), (u32, u32)),
}

/// Fill shape for a builtin feature.
#[derive(Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FeatureShape {
    /// Fill the entire claimed rectangle.
    #[default]
    Square,
    /// Paint an organic blob with a jittered shoreline inside the rectangle.
    Blob,
}

/// Claims a feature footprint via `rule`: an exact or ranged rectangle, or a
/// square rolled from the feature's classic size range. Returns the rect, or
/// `None` when placement failed. Shared prefix for feature builders.
fn claim_feature_footprint(
    blueprint: &MapBlueprint,
    context: &mut PlaceContext,
    rule: &PlaceRule,
    random_generator: &mut ChaCha8Rng,
    feature_name: &str,
    size: Option<FeatureSize>,
    default_min_size: u32,
    default_max_size: u32,
) -> Result<Option<TileRect>, MapError> {
    let footprint = match size {
        Some(FeatureSize::Exact(dimensions)) => UVec2::from(dimensions),
        Some(FeatureSize::Ranged(minimum, maximum)) => UVec2::new(
            random_generator.random_range(minimum.0..=maximum.0),
            random_generator.random_range(minimum.1..=maximum.1),
        ),
        None => UVec2::splat(random_generator.random_range(default_min_size..=default_max_size)),
    };
    let placed_origins = place(rule, footprint, &blueprint.tiles, context, random_generator)?;
    let Some(feature_origin) = placed_origins.first().copied() else {
        return Ok(None);
    };
    TileRect::new(feature_origin, footprint)
        .map(Some)
        .ok_or_else(|| format!("invalid {feature_name} bounds").into())
}

/// Generates an enclosed dead-zone square of 3–10 tiles, including its wall.
/// Returns the placed square's center tile, or `None` when placement failed.
pub fn build_empty_square(
    blueprint: &mut MapBlueprint,
    context: &mut PlaceContext,
    rule: &PlaceRule,
    size: Option<FeatureSize>,
    random_generator: &mut ChaCha8Rng,
) -> Result<Option<IVec2>, MapError> {
    let Some(square_rect) = claim_feature_footprint(
        blueprint,
        context,
        rule,
        random_generator,
        "dead-zone",
        size,
        3,
        10,
    )?
    else {
        return Ok(None);
    };
    let square_origin = square_rect.min;
    if !blueprint.tiles.fill_rect(&square_rect, TileKind::DeadZone) {
        return Err("dead-zone bounds out of map".into());
    }

    let square_maximum = square_rect.max_exclusive() - IVec2::ONE;
    for x in square_origin.x..=square_maximum.x {
        blueprint
            .tiles
            .set(IVec2::new(x, square_origin.y), TileKind::Wall);
        blueprint
            .tiles
            .set(IVec2::new(x, square_maximum.y), TileKind::Wall);
    }

    for y in square_origin.y..=square_maximum.y {
        blueprint
            .tiles
            .set(IVec2::new(square_origin.x, y), TileKind::Wall);
        blueprint
            .tiles
            .set(IVec2::new(square_maximum.x, y), TileKind::Wall);
    }

    Ok(Some(square_rect.center_tile()))
}

/// Paints a 3–8 tile water feature with no walls: a rendered hazard that blocks
/// ground movement and spawns, and merges into colliders automatically. `Blob`
/// shape paints an organic shoreline instead of a square.
/// Returns the pond's center tile, or `None` when placement failed.
pub fn build_pond(
    blueprint: &mut MapBlueprint,
    context: &mut PlaceContext,
    rule: &PlaceRule,
    size: Option<FeatureSize>,
    shape: FeatureShape,
    random_generator: &mut ChaCha8Rng,
) -> Result<Option<IVec2>, MapError> {
    let Some(pond_rect) = claim_feature_footprint(
        blueprint,
        context,
        rule,
        random_generator,
        "pond",
        size,
        3,
        8,
    )?
    else {
        return Ok(None);
    };
    match shape {
        FeatureShape::Square => {
            if !blueprint.tiles.fill_rect(&pond_rect, TileKind::Water) {
                return Err("pond bounds out of map".into());
            }
        }
        FeatureShape::Blob => {
            paint_blob(
                &mut blueprint.tiles,
                pond_rect,
                random_generator,
                TileKind::Water,
            );
        }
    }

    Ok(Some(pond_rect.center_tile()))
}

/// Scatters wall rubble via the entry's rule: use the entry's `count` for scattered
/// coverage, or set `size` for a solid rubble chunk.
/// Returns the chunk's center tile, or `None` when placement failed.
pub fn build_rubble(
    blueprint: &mut MapBlueprint,
    context: &mut PlaceContext,
    rule: &PlaceRule,
    size: Option<FeatureSize>,
    random_generator: &mut ChaCha8Rng,
) -> Result<Option<IVec2>, MapError> {
    let Some(rubble_rect) = claim_feature_footprint(
        blueprint,
        context,
        rule,
        random_generator,
        "rubble",
        size,
        1,
        1,
    )?
    else {
        return Ok(None);
    };
    if !blueprint.tiles.fill_rect(&rubble_rect, TileKind::Wall) {
        return Err("rubble bounds out of map".into());
    }
    Ok(Some(rubble_rect.center_tile()))
}

/// Paints an organic blob inside `rect`: tiles whose normalized elliptical
/// distance from the center are within one, plus shoreline jitter.
fn paint_blob(
    tiles: &mut TileGrid,
    rect: TileRect,
    random_generator: &mut ChaCha8Rng,
    tile: TileKind,
) {
    let half_extents = rect.size.as_vec2() / 2.0;
    let center = rect.min.as_vec2() + half_extents;
    let maximum = rect.max_exclusive();
    for x in rect.min.x..maximum.x {
        for y in rect.min.y..maximum.y {
            let tile_center = IVec2::new(x, y).as_vec2() + Vec2::splat(0.5) - center;
            let normalized = tile_center / (half_extents + Vec2::splat(0.5));
            let shoreline = random_generator.random_range(-0.25..0.25);
            if normalized.length_squared() <= 1.0 + shoreline {
                tiles.set(IVec2::new(x, y), tile);
            }
        }
    }
}
