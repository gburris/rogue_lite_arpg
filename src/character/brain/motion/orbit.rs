//! Orbiting the target inside a distance range.

use bevy::prelude::*;
use rand::RngExt;

use super::{BlockedTimer, SteerInput, SteeredCharacter, Steering};
use crate::character::brain::{Interval, Motion};

const ORBIT_REROLL_INTERVAL: Interval = Interval::new(1.5, 2.5);
const ORBIT_SWITCH_CHANCE: f32 = 0.5;
const ORBIT_RADIAL_GAIN: f32 = 0.5;
/// Seconds an orbit pushes against a wall or a crowd before circling back the other way.
const ORBIT_BLOCKED_SECONDS: f32 = 0.25;
/// Extra seconds a turned-back orbit may stay blocked, so a slow turner gets clear first.
const ORBIT_TURN_BACK_SECONDS: f32 = 0.5;

/// Which way an [`Orbit`] circles its target.
#[derive(Clone, Copy, Debug)]
enum OrbitDirection {
    Clockwise,
    CounterClockwise,
}

impl OrbitDirection {
    fn from_heading(bearing: Vec2, heading: Vec2) -> Self {
        let heading_side = bearing.perp().dot(heading);
        if heading_side > f32::EPSILON {
            Self::Clockwise
        } else if heading_side < -f32::EPSILON {
            Self::CounterClockwise
        } else if rand::rng().random_bool(0.5) {
            // Standing still or heading straight along the bearing favors neither side.
            Self::Clockwise
        } else {
            Self::CounterClockwise
        }
    }

    fn reversed(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

/// Circles the target inside a distance range, rerolling its direction over time and turning
/// back when something blocks the way.
#[derive(Component, Clone, Copy, Debug)]
pub struct Orbit {
    reroll_interval: Interval,
    switch_chance: f32,
    radial_gain: f32,
    max_distance_offset: f32,
    distance_offset: f32,
    reroll_at: f32,
    direction: Option<OrbitDirection>,
    blocked_timer: BlockedTimer,
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
            direction: None,
            blocked_timer: BlockedTimer::default(),
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
        self.direction = None;
        self.blocked_timer.reset();
    }

    fn roll_distance_offset(&mut self) {
        if self.max_distance_offset > 0.0 {
            self.distance_offset =
                rand::rng().random_range(-self.max_distance_offset..self.max_distance_offset);
        }
    }

    fn pick_steering_direction(
        &mut self,
        input: &SteerInput,
        distance_range: Interval,
    ) -> Option<Vec2> {
        let info = input.target_info?;
        let bearing = input.direction_to_target()?;
        let now = input.now;
        let direction = match self.direction {
            None => {
                let direction = OrbitDirection::from_heading(bearing, input.heading);
                self.roll_distance_offset();
                self.reroll_at = now + self.reroll_interval.min();
                direction
            }
            Some(direction)
                if input.tick_blocked_timer(&mut self.blocked_timer, ORBIT_BLOCKED_SECONDS) =>
            {
                self.blocked_timer.delay_by(ORBIT_TURN_BACK_SECONDS);
                self.reroll_at = now + self.reroll_interval.pick_random_value();
                direction.reversed()
            }
            Some(direction) if now >= self.reroll_at => {
                let switched = if rand::rng().random::<f32>() < self.switch_chance {
                    direction.reversed()
                } else {
                    direction
                };
                self.roll_distance_offset();
                self.reroll_at = now + self.reroll_interval.pick_random_value();
                switched
            }
            Some(direction) => direction,
        };
        self.direction = Some(direction);
        // `bearing.perp()` circles the target clockwise.
        let tangent = match direction {
            OrbitDirection::Clockwise => bearing.perp(),
            OrbitDirection::CounterClockwise => -bearing.perp(),
        };
        // Positive error means too far out, so the correction points at the target.
        let radial = bearing
            * distance_range.center_error(info.distance - self.distance_offset)
            * self.radial_gain;
        Some((tangent + radial).normalize_or_zero())
    }
}

impl Default for Orbit {
    fn default() -> Self {
        Self::new()
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
                Steering::scaled(orbit.pick_steering_direction(&input, distance_range), 1.0)
            });
    }
}
