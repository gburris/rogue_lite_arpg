use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::BTreeMap;

use super::blueprint::MapError;
use super::tiles::{TileGrid, TileRect};

/// Inclusive bounds for randomly rolling an element count or footprint size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct CountRange(
    /// Inclusive minimum value.
    pub u32,
    /// Inclusive maximum value.
    pub u32,
);

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
    /// `separation` is the minimum tile distance from every point placed so far.
    Random {
        count: CountRange,
        x_deciles: (u32, u32),
        separation: u32,
    },
    /// Place an inclusive random count of footprints centered within `radius` tiles (a square)
    /// of a named structure's center plus `offset`.
    /// `separation` is the minimum tile distance from every point placed so far.
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
    let bounds = match anchor {
        Anchor::Center => TileRect::centered_in(context.map_size, footprint),
        Anchor::CenterOffset(offset) => TileRect::centered_in(context.map_size, footprint)
            .and_then(|rect| rect.translated(*offset)),
        Anchor::Structure { name, offset } => context
            .structure_centers_by_name
            .get(name)
            .and_then(|center| TileRect::new(*center + *offset - footprint.as_ivec2() / 2, footprint)),
        Anchor::MapMin(offset) => TileRect::new(*offset, footprint),
        Anchor::MapMax(offset) => TileRect::new(
            context.map_size.as_ivec2() - IVec2::ONE + *offset,
            footprint,
        ),
    }
    .ok_or_else(|| {
        format!(
            "fixed footprint {footprint:?} at {anchor:?} is out of bounds for a {}x{} map or references a missing structure",
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
    for _ in 0..target_count {
        if candidates.is_empty() {
            break;
        }
        let candidate_offset = random_generator.random_range(0..candidates.len());
        if let Some(position) = candidates
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
        {
            let candidate_rect =
                TileRect::new(position, footprint).expect("candidate rectangle validated above");
            context.reserved.push(candidate_rect);
            positions.push(position);
        }
    }
    positions
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

#[cfg(test)]
mod placement_tests {
    use super::*;
    use crate::world::map::tiles::TileKind;
    use rand::SeedableRng;

    #[test]
    fn random_placement_skips_footprints_larger_than_the_interior() {
        let tiles = TileGrid::new(UVec2::new(8, 8), TileKind::Grass);
        let mut context = PlaceContext::new(UVec2::new(8, 8));
        let mut random_generator = ChaCha8Rng::seed_from_u64(42);

        let placed = place(
            &PlaceRule::Random {
                count: CountRange(1, 1),
                x_deciles: (1, 9),
                separation: 0,
            },
            UVec2::new(9, 2),
            &tiles,
            &mut context,
            &mut random_generator,
        )
        .unwrap();

        assert!(placed.is_empty());
        assert!(context.reserved.is_empty());
    }

    #[test]
    fn fixed_placement_does_not_consume_rng() {
        let tiles = TileGrid::new(UVec2::new(10, 10), TileKind::Grass);
        let mut context = PlaceContext::new(UVec2::new(10, 10));
        let mut random_generator = ChaCha8Rng::seed_from_u64(42);
        let original_random_generator = random_generator.clone();

        place(
            &PlaceRule::Fixed(Anchor::MapMin(IVec2::new(2, 2))),
            UVec2::ONE,
            &tiles,
            &mut context,
            &mut random_generator,
        )
        .unwrap();

        assert_eq!(random_generator, original_random_generator);
    }

    fn context_with_anchor(center: IVec2) -> (TileGrid, PlaceContext) {
        let tiles = TileGrid::new(UVec2::new(40, 40), TileKind::Grass);
        let mut context = PlaceContext::new(UVec2::new(40, 40));
        context.record_structure_center("Anchor".to_owned(), center);
        (tiles, context)
    }

    #[test]
    fn structure_anchor_offset_is_measured_between_centers() {
        let (tiles, mut context) = context_with_anchor(IVec2::new(20, 20));
        let footprint = UVec2::new(5, 3);
        let mut random_generator = ChaCha8Rng::seed_from_u64(7);

        let origin = place(
            &PlaceRule::Fixed(Anchor::Structure {
                name: "Anchor".to_owned(),
                offset: IVec2::new(4, -6),
            }),
            footprint,
            &tiles,
            &mut context,
            &mut random_generator,
        )
        .unwrap()[0];

        let placed = TileRect::new(origin, footprint).unwrap();
        assert_eq!(placed.center_tile(), IVec2::new(24, 14));
    }

    #[test]
    fn near_structure_with_zero_radius_centers_the_footprint() {
        let (tiles, mut context) = context_with_anchor(IVec2::new(20, 20));
        let footprint = UVec2::new(7, 7);
        let mut random_generator = ChaCha8Rng::seed_from_u64(7);

        let origins = place(
            &PlaceRule::NearStructure {
                count: CountRange(1, 1),
                structure: "Anchor".to_owned(),
                offset: IVec2::new(0, -10),
                radius: 0,
                separation: 0,
            },
            footprint,
            &tiles,
            &mut context,
            &mut random_generator,
        )
        .unwrap();

        let placed = TileRect::new(origins[0], footprint).unwrap();
        assert_eq!(placed.center_tile(), IVec2::new(20, 10));
    }

    #[test]
    fn separation_counts_points_from_earlier_batches() {
        let tiles = TileGrid::new(UVec2::new(12, 12), TileKind::Grass);
        let mut context = PlaceContext::new(UVec2::new(12, 12));
        context.points.push(IVec2::new(6, 6));
        context.keep_clear.push((IVec2::new(1, 1), 3));
        let mut random_generator = ChaCha8Rng::seed_from_u64(3);

        let placed = place(
            &PlaceRule::Random {
                count: CountRange(20, 20),
                x_deciles: (0, 10),
                separation: 4,
            },
            UVec2::ONE,
            &tiles,
            &mut context,
            &mut random_generator,
        )
        .unwrap();

        assert!(!placed.is_empty());
        for tile in placed {
            assert!(tile.distance_squared(IVec2::new(6, 6)) >= 16);
            assert!(tile.distance_squared(IVec2::new(1, 1)) >= 9);
        }
    }
}
