//! Stateless steering toward or away from the target and the anchor.

use bevy::prelude::*;

use super::wander::Wander;
use super::{ANCHOR_ARRIVAL_RADIUS, SteerInput};
use crate::character::brain::{BrainRng, Interval};

const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Straight at the target, when it has a usable bearing.
pub(super) fn chase(input: &SteerInput) -> Option<Vec2> {
    let direction = input.target_info.map_or(Vec2::ZERO, |info| info.direction);
    (direction.length_squared() > f32::EPSILON).then_some(direction)
}

/// Straight away from the target.
pub(super) fn flee(input: &SteerInput) -> Option<Vec2> {
    chase(input).map(|direction| -direction)
}

/// Toward the anchor until the arrival radius.
pub(super) fn home(input: &SteerInput) -> Option<Vec2> {
    let anchor = input.anchor?;
    let to_anchor = anchor.origin() - input.position;
    (to_anchor.length() > ANCHOR_ARRIVAL_RADIUS).then_some(to_anchor.normalize_or_zero())
}

/// Holds the band; a small deadband keeps the arrival from flickering.
pub(super) fn keep_distance(
    input: &SteerInput,
    band: Interval,
    wander: Option<&mut Wander>,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let distance = input.target_info.map_or(0.0, |info| info.distance);
    let direction = input.target_info.map_or(Vec2::ZERO, |info| info.direction);
    if direction.length_squared() <= f32::EPSILON {
        if distance < band.min() {
            return Some(wander?.jitter(input.now, random));
        }
        return None;
    }
    let error = band.error(distance);
    if error.abs() < KEEP_ARRIVAL_DEADBAND {
        None
    } else if error < 0.0 {
        Some(-direction)
    } else {
        Some(direction)
    }
}
