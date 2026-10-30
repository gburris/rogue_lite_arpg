//! Orbiting the target inside a distance range.

use bevy::prelude::*;
use rand::RngExt;

use super::{SteerInput, SteeredCharacter, Steering};
use crate::character::brain::{Capability, CapabilityProfile, Interval, Motion};

const ORBIT_REROLL_INTERVAL: Interval = Interval::new(1.5, 2.5);
const ORBIT_SWITCH_CHANCE: f32 = 0.5;
const ORBIT_RADIAL_GAIN: f32 = 0.5;

/// Circles the target inside a distance range, rerolling its direction over time.
#[derive(Component, Clone, Copy, Debug)]
pub struct Orbit {
    pub reroll_interval: Interval,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub max_distance_offset: f32,
    distance_offset: f32,
    reroll_at: f32,
    clockwise: Option<bool>,
}

impl Orbit {
    pub fn new() -> Self {
        Self {
            reroll_interval: ORBIT_REROLL_INTERVAL,
            switch_chance: ORBIT_SWITCH_CHANCE,
            radial_gain: ORBIT_RADIAL_GAIN,
            max_distance_offset: 0.0,
            distance_offset: 0.0,
            reroll_at: 0.0,
            clockwise: None,
        }
    }

    pub fn with_max_distance_offset(self, max_distance_offset: f32) -> Self {
        Self {
            max_distance_offset,
            ..self
        }
    }

    /// Forgets the circling side, so the next orbit picks it from the heading again.
    fn leave_orbit(&mut self) {
        self.clockwise = None;
    }

    fn roll_distance_offset(&mut self) {
        if self.max_distance_offset > 0.0 {
            self.distance_offset =
                rand::rng().random_range(-self.max_distance_offset..self.max_distance_offset);
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
        Motion::Orbit(Interval::new(0.0, 1.0))
    }

    fn capability_profile(&self) -> CapabilityProfile {
        CapabilityProfile::default()
    }
}

/// Circles the target while the brain holds [`Motion::Orbit`].
pub(super) fn apply_orbit(
    time: Res<Time<Virtual>>,
    mut orbiters: Query<(SteeredCharacter, &mut Orbit, &mut Steering)>,
) {
    for (character, mut orbit, mut steering) in &mut orbiters {
        let Some(motion @ Motion::Orbit(distance_range)) = character.brain.motion() else {
            orbit.leave_orbit();
            continue;
        };
        *steering = character
            .build_steer_input(motion, &time)
            .map_or(Steering::HALT, |input| {
                Steering::scaled(steer(&input, &mut orbit, distance_range), 1.0)
            });
    }
}

fn steer(input: &SteerInput, orbit: &mut Orbit, distance_range: Interval) -> Option<Vec2> {
    let info = input.target_info?;
    let bearing = input.direction_to_target()?;
    let now = input.now;
    let clockwise = match orbit.clockwise {
        None => {
            let clockwise = pick_side_matching_heading(bearing, input.heading);
            orbit.roll_distance_offset();
            orbit.reroll_at = now + orbit.reroll_interval.min();
            clockwise
        }
        Some(clockwise) if now >= orbit.reroll_at => {
            let switched = clockwise ^ (rand::rng().random::<f32>() < orbit.switch_chance);
            orbit.roll_distance_offset();
            orbit.reroll_at = now
                + rand::rng()
                    .random_range(orbit.reroll_interval.min()..orbit.reroll_interval.max());
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
    let radial = bearing
        * distance_range.center_error(info.distance - orbit.distance_offset)
        * orbit.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}

fn pick_side_matching_heading(bearing: Vec2, heading: Vec2) -> bool {
    let heading_side = bearing.perp().dot(heading);
    if heading_side.abs() > f32::EPSILON {
        heading_side > 0.0
    } else {
        // Standing still or heading straight along the bearing favors neither side.
        rand::rng().random_bool(0.5)
    }
}
