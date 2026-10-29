use std::collections::BTreeMap;

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;

use super::blueprint::MapError;
use super::tiles::{TileGrid, TileRect};

/// Inclusive bounds for randomly rolling an element count or footprint size.
/// Recipes spell it as a bare number (`count: 1`) or an inclusive pair (`count: (10, 15)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "CountRangeInput")]
pub struct CountRange(
    /// Inclusive minimum value.
    pub u32,
    /// Inclusive maximum value.
    pub u32,
);

/// Recipe spelling of a [`CountRange`].
#[derive(Deserialize)]
#[serde(untagged)]
enum CountRangeInput {
    Exact(u32),
    Range(u32, u32),
}

impl From<CountRangeInput> for CountRange {
    fn from(input: CountRangeInput) -> Self {
        match input {
            CountRangeInput::Exact(count) => Self(count, count),
            CountRangeInput::Range(minimum, maximum) => Self(minimum, maximum),
        }
    }
}

impl CountRange {
    /// Uniformly rolls from `min..=max`; callers must validate that `min <= max`.
    pub fn roll_size(self, random_generator: &mut ChaCha8Rng) -> u32 {
        random_generator.random_range(self.0..=self.1)
    }
}

/// Selects a footprint origin in tile coordinates.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum Anchor {
    /// Center the footprint on the map.
    Center,
    /// Center on the map, then offset the footprint origin by this many tiles.
    CenterOffset(IVec2),
    /// Center the footprint on a previously placed structure's center, plus `offset` tiles.
    Structure { name: String, offset: IVec2 },
    /// Use this offset from the map's minimum (bottom-left) tile as the footprint origin.
    MapMin(IVec2),
    /// Offset from the map's maximum (top-right) tile; for example `MapMax((-6, -6))`
    /// sits six tiles inside the top-right border for a 1x1 footprint.
    MapMax(IVec2),
}

/// Map edge whose interior row or column should hold the footprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// Placement strategy shared by structures, features, and map entities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceRule {
    /// Place once at the given anchor; does not advance the RNG.
    Fixed(Anchor),
    /// Try to place an inclusive random count within the half-open x-decile interval.
    Random {
        count: CountRange,
        x_deciles: (u32, u32),
        separation: u32,
    },
    /// Place an inclusive random count of footprints centered within `radius` tiles (a square)
    /// of a named structure's center plus `offset`.
    NearStructure {
        count: CountRange,
        structure: String,
        offset: IVec2,
        radius: u32,
        separation: u32,
    },
    /// Place an inclusive random count alongside the selected edge, one tile inside the border.
    Edge { count: CountRange, side: Side },
}

/// Tracks reserved footprints and named structure centers during map building.
#[derive(Clone, Debug, Default)]
pub struct PlaceContext {
    /// Dimensions of the map being populated.
    pub map_size: UVec2,
    /// Footprint rectangles reserved against overlap.
    pub reserved: Vec<TileRect>,
    /// Structure centers indexed by structure name.
    pub structure_centers_by_name: BTreeMap<String, IVec2>,
    /// Every point placement so far (player start, exits, enemies, chests and structure
    /// glyphs). Random and near-structure `separation` is measured against all of them.
    pub points: Vec<IVec2>,
    /// Circles that sampled and edge placements must stay outside of, as `(center, radius)`.
    pub keep_clear: Vec<(IVec2, u32)>,
}

impl PlaceContext {
    /// Creates empty placement state for a map of this size.
    pub fn new(map_size: UVec2) -> Self {
        Self {
            map_size,
            ..default()
        }
    }

    /// Records the named structure center for later anchored placements.
    pub fn record_structure_center(&mut self, name: String, center: IVec2) {
        self.structure_centers_by_name.insert(name, center);
    }

    /// Returns whether `tile` lies outside every keep-clear circle.
    pub fn is_clear(&self, tile: IVec2) -> bool {
        self.keep_clear
            .iter()
            .all(|(center, radius)| center.distance_squared(tile) >= (*radius as i32).pow(2))
    }

    fn separated(&self, tile: IVec2, separation: u32) -> bool {
        separation == 0
            || self
                .points
                .iter()
                .all(|point| point.distance_squared(tile) >= (separation as i32).pow(2))
    }

    fn available(&self, tiles: &TileGrid, rect: TileRect) -> bool {
        tiles.contains_rect(&rect)
            && tiles
                .rect_iter(&rect)
                .is_some_and(|mut iter| iter.all(|(_, tile_kind)| tile_kind.can_spawn()))
            && !self
                .reserved
                .iter()
                .any(|reserved_rect| overlaps(*reserved_rect, rect))
    }
}

fn overlaps(first_rect: TileRect, second_rect: TileRect) -> bool {
    first_rect.min.x < second_rect.max_exclusive().x
        && second_rect.min.x < first_rect.max_exclusive().x
        && first_rect.min.y < second_rect.max_exclusive().y
        && second_rect.min.y < first_rect.max_exclusive().y
}

/// Places footprints and returns their origins; random and edge rules may place fewer than requested.
/// Successful placements update context; fixed placement does not advance the RNG.
pub fn place(
    rule: &PlaceRule,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> Result<Vec<IVec2>, MapError> {
    match rule {
        PlaceRule::Fixed(anchor) => place_fixed(anchor, footprint, tiles, context),
        PlaceRule::Random {
            count,
            x_deciles,
            separation,
        } => Ok(place_random(
            *count,
            *x_deciles,
            *separation,
            footprint,
            tiles,
            context,
            random_generator,
        )),
        PlaceRule::NearStructure {
            count,
            structure,
            offset,
            radius,
            separation,
        } => place_near_structure(
            *count,
            structure,
            *offset,
            *radius,
            *separation,
            footprint,
            tiles,
            context,
            random_generator,
        ),
        PlaceRule::Edge { count, side } => Ok(place_edge(
            *count,
            *side,
            footprint,
            tiles,
            context,
            random_generator,
        )),
    }
}

fn place_fixed(
    anchor: &Anchor,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
) -> Result<Vec<IVec2>, MapError> {
    let bounds =
        match anchor {
            Anchor::Center => TileRect::centered_in(context.map_size, footprint),
            Anchor::CenterOffset(offset) => TileRect::centered_in(context.map_size, footprint)
                .and_then(|rect| rect.translated(*offset)),
            Anchor::Structure { name, offset } => context
                .structure_centers_by_name
                .get(name)
                .and_then(|center| {
                    TileRect::new(*center + *offset - footprint.as_ivec2() / 2, footprint)
                }),
            Anchor::MapMin(offset) => TileRect::new(*offset, footprint),
            Anchor::MapMax(offset) => TileRect::new(
                context.map_size.as_ivec2() - IVec2::ONE + *offset,
                footprint,
            ),
        }
        .ok_or_else(|| {
            format!(
                "fixed footprint {footprint:?} at {anchor:?} is out of bounds for a {}x{} map \
             or references a missing structure",
                context.map_size.x, context.map_size.y
            )
        })?;
    if !context.available(tiles, bounds) {
        return Err(format!(
            "fixed footprint {bounds:?} at {anchor:?} overlaps a reserved area or covers unspawnable tiles"
        )
        .into());
    }
    context.reserved.push(bounds);
    Ok(vec![bounds.min])
}

fn place_random(
    count: CountRange,
    x_deciles: (u32, u32),
    separation: u32,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> Vec<IVec2> {
    let target_count = count.roll_size(random_generator);
    let x_start = context.map_size.x * x_deciles.0 / 10;
    let x_end = context.map_size.x * x_deciles.1 / 10;
    if x_start >= x_end
        || context.map_size.x <= footprint.x.saturating_add(2)
        || context.map_size.y <= footprint.y.saturating_add(2)
    {
        return Vec::new();
    }
    // Guarded above: `map_size.y - 1 >= 2`, so this range is non-empty.
    let y_end = context.map_size.y - 1;
    scatter_footprints(
        target_count,
        separation,
        footprint,
        tiles,
        context,
        random_generator,
        |random_generator| {
            IVec2::new(
                random_generator.random_range(x_start..x_end) as i32,
                random_generator.random_range(1..y_end) as i32,
            )
        },
    )
}

fn place_near_structure(
    count: CountRange,
    structure: &str,
    offset: IVec2,
    radius: u32,
    separation: u32,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> Result<Vec<IVec2>, MapError> {
    let target_count = count.roll_size(random_generator);
    let Some(structure_center) = context.structure_centers_by_name.get(structure).copied() else {
        return Err(format!("missing structure anchor {structure}").into());
    };
    // Sample the footprint's center, then shift to its bottom-left origin.
    let target_center = structure_center + offset;
    let half_footprint = footprint.as_ivec2() / 2;
    let radius_tiles = i32::try_from(radius).map_err(|_| "placement radius is too large")?;
    if context.map_size.x <= footprint.x.saturating_add(2)
        || context.map_size.y <= footprint.y.saturating_add(2)
    {
        return Ok(Vec::new());
    }
    Ok(scatter_footprints(
        target_count,
        separation,
        footprint,
        tiles,
        context,
        random_generator,
        |random_generator| {
            target_center - half_footprint
                + IVec2::new(
                    random_generator.random_range(-radius_tiles..=radius_tiles),
                    random_generator.random_range(-radius_tiles..=radius_tiles),
                )
        },
    ))
}

fn place_edge(
    count: CountRange,
    side: Side,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> Vec<IVec2> {
    let target_count = count.roll_size(random_generator);
    let mut positions = Vec::new();
    let maximum_origin = context.map_size.as_ivec2() - IVec2::ONE - footprint.as_ivec2();
    if maximum_origin.x < 1 || maximum_origin.y < 1 {
        return positions;
    }
    let (candidate_start, candidate_end) = match side {
        Side::Left | Side::Right => (1, maximum_origin.y),
        Side::Top | Side::Bottom => (1, maximum_origin.x),
    };
    let candidates = (candidate_start..=candidate_end)
        .map(|coordinate| match side {
            Side::Left => IVec2::new(1, coordinate),
            Side::Right => IVec2::new(maximum_origin.x, coordinate),
            Side::Top => IVec2::new(coordinate, maximum_origin.y),
            Side::Bottom => IVec2::new(coordinate, 1),
        })
        .collect::<Vec<_>>();
    // Split the wall into one segment per footprint so several exits spread out instead of
    // landing on adjacent rows. A blocked segment falls back to the whole wall.
    let segment_count = target_count as usize;
    for segment_index in 0..segment_count {
        let segment = &candidates[segment_index * candidates.len() / segment_count
            ..(segment_index + 1) * candidates.len() / segment_count];
        let position = pick_edge_candidate(segment, footprint, tiles, context, random_generator)
            .or_else(|| {
                pick_edge_candidate(&candidates, footprint, tiles, context, random_generator)
            });
        if let Some(position) = position {
            let candidate_rect =
                TileRect::new(position, footprint).expect("candidate rectangle validated above");
            context.reserved.push(candidate_rect);
            positions.push(position);
        }
    }
    positions
}

/// Starting from a random candidate, returns the first one (wrapping) that is free and clear.
fn pick_edge_candidate(
    candidates: &[IVec2],
    footprint: UVec2,
    tiles: &TileGrid,
    context: &PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> Option<IVec2> {
    if candidates.is_empty() {
        return None;
    }
    let candidate_offset = random_generator.random_range(0..candidates.len());
    candidates
        .iter()
        .cycle()
        .skip(candidate_offset)
        .take(candidates.len())
        .copied()
        .find(|position| {
            context.is_clear(*position)
                && TileRect::new(*position, footprint)
                    .is_some_and(|candidate_rect| context.available(tiles, candidate_rect))
        })
}

/// Retries sampled origins until `target_count` footprints are claimed or attempts run out.
/// Claimed footprints are reserved in the context; fewer may be returned when space runs out.
/// Candidates must keep `separation` from this batch and from every recorded point, and
/// stay outside every keep-clear circle.
fn scatter_footprints(
    target_count: u32,
    separation: u32,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
    mut sample_origin: impl FnMut(&mut ChaCha8Rng) -> IVec2,
) -> Vec<IVec2> {
    let mut positions = Vec::new();
    for _ in 0..(target_count as usize * 100) {
        if positions.len() >= target_count as usize {
            break;
        }
        let candidate_origin = sample_origin(random_generator);
        let Some(candidate_rect) = TileRect::new(candidate_origin, footprint) else {
            continue;
        };
        if context.available(tiles, candidate_rect)
            && context.is_clear(candidate_origin)
            && context.separated(candidate_origin, separation)
            && positions.iter().all(|existing_origin: &IVec2| {
                existing_origin.distance_squared(candidate_origin) >= (separation as i32).pow(2)
            })
        {
            context.reserved.push(candidate_rect);
            positions.push(candidate_origin);
        }
    }
    positions
}
