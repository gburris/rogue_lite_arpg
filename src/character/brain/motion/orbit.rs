//! Orbiting the target inside a distance range.

use bevy::prelude::*;

use super::{Capability, SteerInput};
use crate::character::brain::{BrainRng, Interval, Motion, Tuning};

const ORBIT_REROLL_MIN: f32 = 1.5;
const ORBIT_REROLL_MAX: f32 = 2.5;
const ORBIT_SWITCH_CHANCE: f32 = 0.5;
const ORBIT_RADIAL_GAIN: f32 = 0.5;

/// Circles the target inside a distance range, rerolling its direction over time. Opt-in at spawn.
#[derive(Component, Clone, Copy, Debug)]
pub struct Orbit {
    pub reroll_min: f32,
    pub reroll_max: f32,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub standoff_range: f32,
    standoff: f32,
    reroll_at: f32,
    clockwise: Option<bool>,
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
            clockwise: None,
        }
    }

    pub fn with_standoff(standoff_range: f32) -> Self {
        Self {
            standoff_range,
            ..Self::new()
        }
    }

    /// Forgets the circling side, so the next orbit picks it from the heading again.
    pub(super) fn leave_orbit(&mut self) {
        self.clockwise = None;
    }

    fn roll_standoff(&mut self, random: &mut BrainRng) {
        if self.standoff_range > 0.0 {
            self.standoff = random.range(-self.standoff_range, self.standoff_range);
        }
    }
}

impl Default for Orbit {
    fn default() -> Self {
        Self::new()
    }
}

/// Makes `Motion::Orbit` available only to a character that carries one.
impl Capability for Orbit {
    type Key = Motion;

    fn key(&self) -> Motion {
        // Tables index by variant, so the range only stands in.
        Motion::Orbit(Interval::new(0.0, 1.0))
    }

    fn tuning(&self) -> Tuning {
        Tuning::default()
    }
}

pub(super) fn steer(
    input: &SteerInput,
    orbit: &mut Orbit,
    distance_range: Interval,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let info = input.target_info?;
    let bearing = input.direction_to_target()?;
    let now = input.now;
    let clockwise = match orbit.clockwise {
        None => {
            let clockwise = pick_side_matching_heading(bearing, input.heading, random);
            orbit.roll_standoff(random);
            orbit.reroll_at = now + orbit.reroll_min;
            clockwise
        }
        Some(clockwise) if now >= orbit.reroll_at => {
            let switched = clockwise ^ (random.range(0.0, 1.0) < orbit.switch_chance);
            orbit.roll_standoff(random);
            orbit.reroll_at = now + random.range(orbit.reroll_min, orbit.reroll_max);
            switched
        }
        Some(clockwise) => clockwise,
    };
    orbit.clockwise = Some(clockwise);
    // `bearing.perp()` circles the target clockwise.
    let tangent = if clockwise {
        bearing.perp()
    } else {
        -bearing.perp()
    };
    // Positive error means too far out, so the correction points at the target.
    let radial =
        bearing * distance_range.center_error(info.distance - orbit.standoff) * orbit.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}

fn pick_side_matching_heading(bearing: Vec2, heading: Vec2, random: &mut BrainRng) -> bool {
    let heading_side = bearing.perp().dot(heading);
    if heading_side.abs() > f32::EPSILON {
        heading_side > 0.0
    } else {
        // Standing still or heading straight along the bearing favors neither side.
        random.range(0.0, 1.0) < 0.5
    }
}
