use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::world::map::{
    blueprint::{MapBlueprint, MapError},
    place::{CountRange, PlaceContext, PlaceRule, place},
    tiles::{TileRect, TileType},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Procedural tile shapes that fill a placed footprint.
pub enum FeatureKind {
    /// A square dead zone whose inclusive rolled size includes its wall border.
    WalledSquare { size: CountRange },
}

impl FeatureKind {
    /// Fills this feature using `rule` to choose the tile-space footprint.
    pub fn fill(
        self,
        blueprint: &mut MapBlueprint,
        context: &mut PlaceContext,
        rule: &PlaceRule,
        rng: &mut ChaCha8Rng,
    ) -> Result<(), MapError> {
        let Self::WalledSquare { size } = self;
        let footprint = UVec2::splat(size.roll_size(rng));
        let positions = place(rule, footprint, &blueprint.tiles, context, rng)?;
        let Some(min) = positions.first().copied() else {
            return Ok(());
        };
        let rect = TileRect::new(min, footprint).ok_or("invalid dead-zone bounds")?;
        if !blueprint.tiles.fill_rect(&rect, TileType::DeadZone) {
            return Err("dead-zone bounds out of map".into());
        }
        let max = rect.max_exclusive();
        let max_tile = max - IVec2::ONE;
        for x in min.x..=max_tile.x {
            blueprint.tiles.set(IVec2::new(x, min.y), TileType::Wall);
            blueprint
                .tiles
                .set(IVec2::new(x, max_tile.y), TileType::Wall);
        }
        for y in min.y..=max_tile.y {
            blueprint.tiles.set(IVec2::new(min.x, y), TileType::Wall);
            blueprint
                .tiles
                .set(IVec2::new(max_tile.x, y), TileType::Wall);
        }
        Ok(())
    }
}

/// Builds a walled-square feature with an inclusive size range of 3 to 10 tiles.
pub fn build_empty_square(
    blueprint: &mut MapBlueprint,
    context: &mut PlaceContext,
    rule: &PlaceRule,
    rng: &mut ChaCha8Rng,
) -> Result<(), MapError> {
    (FeatureKind::WalledSquare {
        size: CountRange(3, 10),
    })
    .fill(blueprint, context, rule, rng)
}
