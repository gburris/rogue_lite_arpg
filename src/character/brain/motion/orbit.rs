//! Orbiting the target inside a distance range.

use bevy::prelude::*;

use super::{OrbitDirection, SteerInput};
use crate::character::brain::{BrainRng, Interval};

const ORBIT_REROLL_MIN: f32 = 1.5;
const ORBIT_REROLL_MAX: f32 = 2.5;
const ORBIT_SWITCH_CHANCE: f32 = 0.5;
const ORBIT_RADIAL_GAIN: f32 = 0.5;

/// Circles the target inside a distance range, rerolling its direction over time. Opt-in at spawn.
#[derive(Component, Clone, Copy, Debug)]
#[require(OrbitDirection)]
pub struct Orbit {
    pub reroll_min: f32,
    pub reroll_max: f32,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub standoff_range: f32,
    standoff: f32,
    reroll_at: f32,
}

impl Orbit {
    pub fn new() -> Self {
        Self {
            reroll_min: ORBIT_REROLL_MIN,
            reroll_max: ORBIT_REROLL_MAX,
            switch_chance: ORBIT_SWITCH_CHANCE,
            radial_gain: ORBIT_RADIAL_GAIN,
            standoff_range: 0.0,
            standoff: 0.0,
            reroll_at: 0.0,
        }
    }

    pub fn with_standoff(standoff_range: f32) -> Self {
        Self {
            standoff_range,
            ..Self::new()
        }
    }
}

impl Default for Orbit {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn steer(
    input: &SteerInput,
    orbit: Option<&mut Orbit>,
    orbit_direction: Option<&mut OrbitDirection>,
    distance_range: Interval,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let info = input.target_info?;
    let bearing = input.direction_to_target()?;
    let orbit = orbit?;
    let orbit_direction = orbit_direction?;
    let now = input.now;
    if now >= orbit.reroll_at {
        if !orbit_direction.is_held(now) {
            orbit_direction.reroll(orbit.switch_chance, random);
            if orbit.standoff_range > 0.0 {
                orbit.standoff = random.range(-orbit.standoff_range, orbit.standoff_range);
            }
        }
        orbit.reroll_at = now + random.range(orbit.reroll_min, orbit.reroll_max);
    }
    let tangent = bearing.perp() * orbit_direction.sign(random);
    // Positive error means too far out, so the correction points at the target.
    let radial =
        bearing * distance_range.center_error(info.distance - orbit.standoff) * orbit.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}
