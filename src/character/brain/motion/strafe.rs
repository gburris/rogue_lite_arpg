//! Orbiting the target inside a distance band.

use bevy::prelude::*;

use super::{SteerInput, approach};
use crate::character::{
    brain::{BrainRng, DistanceBand},
    vision::TargetInfo,
};

const STRAFE_REROLL_MIN: f32 = 1.5;
const STRAFE_REROLL_MAX: f32 = 2.5;
const STRAFE_SWITCH_CHANCE: f32 = 0.5;
const STRAFE_RADIAL_GAIN: f32 = 0.5;
const SMART_STRAFE_REROLL_MIN: f32 = 0.5;
const SMART_STRAFE_REROLL_MAX: f32 = 0.9;
const SMART_STRAFE_INPUT_DEADBAND: f32 = 20.0;

/// How one entity strafes. Opt-in at spawn; absent means never strafes.
#[derive(Component, Clone, Copy, Debug)]
pub struct Strafe {
    pub reroll_min: f32,
    pub reroll_max: f32,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub standoff_range: f32,
    /// Whether the orbit side should follow the target's tangential movement.
    pub smart_orbit: bool,
    standoff: f32,
    orbit: Option<Orbit>,
    reroll_at: f32,
}

impl Strafe {
    /// Orbits the brain's band with the default rhythm and no radius standoff.
    pub fn new() -> Self {
        Self {
            reroll_min: STRAFE_REROLL_MIN,
            reroll_max: STRAFE_REROLL_MAX,
            switch_chance: STRAFE_SWITCH_CHANCE,
            radial_gain: STRAFE_RADIAL_GAIN,
            standoff_range: 0.0,
            smart_orbit: false,
            standoff: 0.0,
            orbit: None,
            reroll_at: 0.0,
        }
    }

    /// Orbits with a short reaction window and counter-moves the target's
    /// tangential motion instead of rerolling the side at random.
    pub fn smart() -> Self {
        Self {
            reroll_min: SMART_STRAFE_REROLL_MIN,
            reroll_max: SMART_STRAFE_REROLL_MAX,
            switch_chance: 0.0,
            smart_orbit: true,
            ..Self::new()
        }
    }

    /// Also rerolls a radius offset in `-standoff_range..=standoff_range`.
    pub fn with_standoff(standoff_range: f32) -> Self {
        Self {
            standoff_range,
            ..Self::new()
        }
    }
}

impl Default for Strafe {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Orbit {
    #[default]
    Clockwise,
    CounterClockwise,
}

impl Orbit {
    fn flipped(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }

    fn sign(self) -> f32 {
        match self {
            Self::Clockwise => -1.0,
            Self::CounterClockwise => 1.0,
        }
    }
}

/// Tangent plus a radial correction toward `band`, which comes from the
/// brain's definition so it cannot drift from the band its gates measure.
/// `hold_orbit` freezes the
/// side and standoff rerolls, used while a dive settles back into the circle
/// so a flip does not reverse the tangent at the handoff.
pub(super) fn steer(
    input: &SteerInput,
    strafe: Option<&mut Strafe>,
    band: Option<DistanceBand>,
    random: &mut BrainRng,
    hold_orbit: bool,
) -> Option<Vec2> {
    let info = input.target_info?;
    let band = band?;
    let direction = approach::chase(input)?;
    let strafe = strafe?;
    let now = input.now;
    let orbit_due = now >= strafe.reroll_at;
    if strafe.orbit.is_none() || (!hold_orbit && orbit_due) {
        let orbit = if strafe.smart_orbit {
            smart_orbit(info, direction, strafe.orbit, random)
        } else {
            match strafe.orbit {
                Some(orbit) if random.range(0.0, 1.0) < strafe.switch_chance => orbit.flipped(),
                Some(orbit) => orbit,
                None if random.range(0.0, 1.0) < 0.5 => Orbit::Clockwise,
                None => Orbit::CounterClockwise,
            }
        };
        strafe.orbit = Some(orbit);
        if strafe.standoff_range > 0.0 {
            strafe.standoff = random.range(-strafe.standoff_range, strafe.standoff_range);
        }
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    } else if hold_orbit && orbit_due {
        // A dive can outlive the orbit's reroll clock. Start a fresh window at
        // the handoff instead of flipping immediately after the recovery.
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    }
    let tangent = direction.perp() * strafe.orbit.unwrap_or_default().sign();
    // Positive error means too far from the shifted ring, so the correction
    // points at the target.
    let radial =
        direction * band.center_error(info.distance - strafe.standoff) * strafe.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}

/// Picks the orbit side that moves against the target's tangential motion.
/// Radial or still targets keep the current side so the mage does not jitter.
fn smart_orbit(
    info: &TargetInfo,
    direction: Vec2,
    current: Option<Orbit>,
    random: &mut BrainRng,
) -> Orbit {
    let tangential_motion = info.target_velocity.dot(direction.perp());
    if tangential_motion.abs() > SMART_STRAFE_INPUT_DEADBAND {
        if tangential_motion > 0.0 {
            Orbit::Clockwise
        } else {
            Orbit::CounterClockwise
        }
    } else if let Some(orbit) = current {
        orbit
    } else if random.range(0.0, 1.0) < 0.5 {
        Orbit::Clockwise
    } else {
        Orbit::CounterClockwise
    }
}
