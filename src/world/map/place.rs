use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

use super::tiles::{TileGrid, TileRect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Inclusive bounds for randomly rolling an element count or footprint size.
pub struct CountRange(pub u32, pub u32);


impl CountRange {
    /// Uniformly rolls from `min..=max`; callers must validate that `min <= max`.
    pub fn roll_size(self, rng: &mut ChaCha8Rng) -> u32 {
        rng.random_range(self.0..=self.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Selects a footprint origin in tile coordinates.
pub enum Anchor {
    /// Center the footprint on the map.
    Center,
    /// Center on the map, then offset the footprint origin by this many tiles.
    CenterOffset(IVec2),
    /// Use this tile as the footprint's minimum (bottom-left) coordinate.
    At(IVec2),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Map edge whose interior row/column should hold the footprint.
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Placement strategy shared by structures, features, and map entities.
pub enum PlaceRule {
    /// Place once at the given anchor; does not advance the RNG.
    Fixed(Anchor),
    /// Try to place an inclusive random count within the half-open x-decile interval.
    /// `separation` is the minimum tile distance between origins returned by this call.
    Random {
        count: CountRange,
        x_deciles: (u32, u32),
        separation: u32,
    },
    /// Place an inclusive random count alongside the selected edge, one tile inside the border.
    Edge {
        count: CountRange,
        side: Side,
    },
}

#[derive(Clone, Debug, Default)]
/// Mutable placement state shared while building one map.
///
/// `map_size` must match `tiles`; reserved rectangles prevent footprint overlap,

/// while `occupied` tracks individual entity tiles. Structure centers are recorded
/// in placement order for later structure-relative anchors.
pub struct PlaceContext {
    pub map_size: UVec2,
    pub reserved: Vec<TileRect>,
    pub occupied: HashSet<IVec2>,
    pub structure_centers: Vec<IVec2>,
}

impl PlaceContext {
    /// Creates empty placement state for a map of this size.
    pub fn new(map_size: UVec2) -> Self {
        Self {
            map_size,
            ..default()
        }
    }

    fn available(&self, tiles: &TileGrid, rect: TileRect) -> bool {
        tiles.contains_rect(&rect)
            && tiles.rect_iter(&rect).is_some_and(|mut iter| {
                iter.all(|(p, t)| t.can_spawn() && !self.occupied.contains(&p))
            })
            && !self.reserved.iter().any(|r| overlaps(*r, rect))
    }
}

fn overlaps(a: TileRect, b: TileRect) -> bool {
    a.min.x < b.max_exclusive().x
        && b.min.x < a.max_exclusive().x
        && a.min.y < b.max_exclusive().y
        && b.min.y < a.max_exclusive().y
}

/// Places a rectangular footprint and returns the minimum tile of each placement.
/// Random and edge rules may return fewer positions than requested when space is scarce.
/// Successful placements update `context`; fixed placement never consumes RNG.
pub fn place(
    rule: &PlaceRule,
    footprint: UVec2,
    tiles: &TileGrid,
    context: &mut PlaceContext,
    rng: &mut ChaCha8Rng,
) -> Result<Vec<IVec2>, super::blueprint::MapError> {
    match *rule {
        PlaceRule::Fixed(anchor) => {
            let bounds = match anchor {
                Anchor::Center => TileRect::centered_in(context.map_size, footprint),
                Anchor::CenterOffset(offset) => TileRect::centered_in(context.map_size, footprint)
                    .and_then(|r| r.translated(offset)),
                Anchor::At(min) => TileRect::new(min, footprint),
            }
            .ok_or_else(|| "fixed footprint out of bounds".to_string())?;
            if !context.available(tiles, bounds) {
                return Err("fixed footprint unavailable".into());
            }
            context.reserved.push(bounds);
            context.structure_centers.push(bounds.center_tile());
            Ok(vec![bounds.min])
        }
        PlaceRule::Random {
            count,
            x_deciles,
            separation,
        } => {
            let amount = count.roll_size(rng);
            let mut out = Vec::new();
            let x0 = context.map_size.x * x_deciles.0 / 10;
            let x1 = context.map_size.x * x_deciles.1 / 10;
            if x0 >= x1 || context.map_size.y <= 2 {
                return Ok(out);
            }
            for _ in 0..(amount as usize * 100) {
                if out.len() >= amount as usize {
                    break;
                }
                let p = IVec2::new(
                    rng.random_range(x0..x1) as i32,
                    rng.random_range(1..context.map_size.y - 1) as i32,
                );
                let rect = TileRect::new(p, footprint).unwrap();
                if context.available(tiles, rect)
                    && out
                        .iter()
                        .all(|q: &IVec2| q.distance_squared(p) >= (separation as i32).pow(2))
                {
                    context.occupied.insert(p);
                    context.reserved.push(rect);
                    context.structure_centers.push(rect.center_tile());
                    out.push(p);
                }
            }
            Ok(out)
        }
        PlaceRule::Edge { count, side } => {
            let amount = count.roll_size(rng);
            let mut out = Vec::new();
            let max_origin = context.map_size.as_ivec2() - IVec2::ONE - footprint.as_ivec2();
            if max_origin.x < 1 || max_origin.y < 1 {
                return Ok(out);
            }
            let (start, end) = match side {
                Side::Left | Side::Right => (1, max_origin.y),
                Side::Top | Side::Bottom => (1, max_origin.x),
            };
            let candidates = (start..=end)
                .map(|v| match side {
                    Side::Left => IVec2::new(1, v),
                    Side::Right => IVec2::new(max_origin.x, v),
                    Side::Top => IVec2::new(v, max_origin.y),
                    Side::Bottom => IVec2::new(v, 1),
                })
                .collect::<Vec<_>>();
            for _ in 0..amount {
                if candidates.is_empty() {
                    break;
                }
                let start = rng.random_range(0..candidates.len());
                if let Some(p) = candidates
                    .iter()
                    .cycle()
                    .skip(start)
                    .take(candidates.len())
                    .copied()
                    .find(|p| context.available(tiles, TileRect::new(*p, footprint).unwrap()))
                {
                    let rect = TileRect::new(p, footprint).unwrap();
                    context.occupied.insert(p);
                    context.reserved.push(rect);
                    context.structure_centers.push(rect.center_tile());
                    out.push(p);
                }
            }
            Ok(out)
        }
    }
}
