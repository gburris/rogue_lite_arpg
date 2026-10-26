//! Orbiting the target inside a distance range.

use bevy::prelude::*;

use super::SteerInput;
use crate::character::brain::{BrainRng, Interval};

const STRAFE_REROLL_MIN: f32 = 1.5;
const STRAFE_REROLL_MAX: f32 = 2.5;
const STRAFE_SWITCH_CHANCE: f32 = 0.5;
const STRAFE_RADIAL_GAIN: f32 = 0.5;

/// Circles the target inside a distance range, rerolling its side over time. Opt-in at spawn.
#[derive(Component, Clone, Copy, Debug)]
pub struct Strafe {
    pub reroll_min: f32,
    pub reroll_max: f32,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub standoff_range: f32,
    standoff: f32,
    orbit: Option<Orbit>,
    reroll_at: f32,
}

impl Strafe {
    pub fn new() -> Self {
        Self {
            reroll_min: STRAFE_REROLL_MIN,
            reroll_max: STRAFE_REROLL_MAX,
            switch_chance: STRAFE_SWITCH_CHANCE,
            radial_gain: STRAFE_RADIAL_GAIN,
            standoff_range: 0.0,
            standoff: 0.0,
            orbit: None,
            reroll_at: 0.0,
        }
    }

    pub fn with_standoff(standoff_range: f32) -> Self {
        Self {
            standoff_range,
            ..Self::new()
        }
    }

    pub(super) fn flip_orbit(&mut self, now: f32, random: &mut BrainRng) -> Orbit {
        let orbit = self
            .orbit
            .map_or_else(|| Orbit::random(random), Orbit::flipped);
        self.orbit = Some(orbit);
        self.reroll_at = now + random.range(self.reroll_min, self.reroll_max);
        orbit
    }
}

impl Default for Strafe {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Orbit {
    #[default]
    Clockwise,
    CounterClockwise,
}

impl Orbit {
    pub(super) fn random(random: &mut BrainRng) -> Self {
        if random.range(0.0, 1.0) < 0.5 {
            Self::Clockwise
        } else {
            Self::CounterClockwise
        }
    }

    fn flipped(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }

    pub(super) fn sign(self) -> f32 {
        match self {
            Self::Clockwise => -1.0,
            Self::CounterClockwise => 1.0,
        }
    }
}

pub(super) fn steer(
    input: &SteerInput,
    strafe: Option<&mut Strafe>,
    distance_range: Interval,
    random: &mut BrainRng,
    hold_orbit: bool,
) -> Option<Vec2> {
    let info = input.target_info?;
    let direction = input.direction_to_target()?;
    let strafe = strafe?;
    let now = input.now;
    let orbit_due = now >= strafe.reroll_at;
    // `hold_orbit` freezes rerolls while a swoop settles back into the circle,
    // so the tangent does not reverse at the handoff.
    if strafe.orbit.is_none() || (!hold_orbit && orbit_due) {
        let orbit = match strafe.orbit {
            Some(orbit) if random.range(0.0, 1.0) < strafe.switch_chance => orbit.flipped(),
            Some(orbit) => orbit,
            None => Orbit::random(random),
        };
        strafe.orbit = Some(orbit);
        if strafe.standoff_range > 0.0 {
            strafe.standoff = random.range(-strafe.standoff_range, strafe.standoff_range);
        }
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    } else if hold_orbit && orbit_due {
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    }
    let tangent = direction.perp() * strafe.orbit.unwrap_or_default().sign();
    // Positive error means too far out, so the correction points at the target.
    let radial = direction
        * distance_range.center_error(info.distance - strafe.standoff)
        * strafe.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}
