use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::world::map::{
    blueprint::{MapBlueprint, MapError},
    place::{PlaceContext, PlaceRule, place},
    structures::{TEMPLE, apply_structure},
};

pub fn build_temple(
    blueprint: &mut MapBlueprint,
    context: &mut PlaceContext,
    rule: &PlaceRule,
    rng: &mut ChaCha8Rng,
) -> Result<(), MapError> {
    let size = TEMPLE.size().as_uvec2();
    let positions = place(rule, size, &blueprint.tiles, context, rng)?;
    if let Some(min) = positions.first() {
        apply_structure(blueprint, &TEMPLE, *min)
            .map_err(|error| format!("could not place temple structure: {error:?}"))?;
    }
    Ok(())
}
