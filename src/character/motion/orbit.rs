use bevy::prelude::*;
use rand::RngExt;

use super::{
    BlockedTimer, CircleDirection, MotionKey, MotionOf, MotionProgress, RunningMotion, SteerInput,
    SteeredCharacter, Steering, SteeringTargets, TunedBy,
};
use crate::interval::{Interval, Spread};

const ORBIT_REROLL_INTERVAL: Interval = Interval::new(1.5, 2.5);
const ORBIT_SWITCH_CHANCE: f32 = 0.5;
const ORBIT_RADIAL_GAIN: f32 = 0.5;
/// Seconds an orbit may stay blocked before it reverses direction.
const ORBIT_BLOCKED_SECONDS: f32 = 0.25;
/// Extra seconds a reversed orbit may stay blocked, so a slow-turning character can get clear.
const ORBIT_TURN_BACK_SECONDS: f32 = 0.5;

/// Circles the target within a distance range. It rerolls its direction over time and reverses
/// when blocked.
#[derive(Component, Clone, Copy, Debug)]
#[require(MotionKey::Orbit)]
pub struct Orbit {
    distance_range: Interval,
    reroll_interval: Interval,
    switch_chance: f32,
    radial_gain: f32,
    distance_offset_spread: Spread,
}

impl Orbit {
    /// Circles within `distance_range` of the target.
    pub fn new(distance_range: Interval) -> Self {
        Self {
            distance_range,
            reroll_interval: ORBIT_REROLL_INTERVAL,
            switch_chance: ORBIT_SWITCH_CHANCE,
            radial_gain: ORBIT_RADIAL_GAIN,
            distance_offset_spread: Spread::default(),
        }
    }

    pub fn with_distance_offset_spread(self, half_width: f32) -> Self {
        Self {
            distance_offset_spread: Spread::new(half_width),
            ..self
        }
    }
}

/// An [`Orbit`] in progress: its direction and when it next rerolls.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct OrbitProgress {
    distance_offset: f32,
    reroll_at: f32,
    direction: Option<CircleDirection>,
    blocked_timer: BlockedTimer,
}

impl OrbitProgress {
    fn roll_distance_offset(&mut self, orbit: &Orbit) {
        self.distance_offset = orbit.distance_offset_spread.pick_random_value();
    }

    fn pick_steering_direction(&mut self, orbit: &Orbit, input: &SteerInput) -> Option<Vec2> {
        let target = input.target?;
        let bearing = input.direction_to_target()?;
        let now = input.now;
        let direction = match self.direction {
            None => {
                let direction = CircleDirection::from_heading(bearing, input.heading);
                self.roll_distance_offset(orbit);
                self.reroll_at = now + orbit.reroll_interval.min();
                direction
            }
            Some(direction)
                if input.tick_blocked_timer(&mut self.blocked_timer, ORBIT_BLOCKED_SECONDS) =>
            {
                self.blocked_timer.delay_by(ORBIT_TURN_BACK_SECONDS);
                self.reroll_at = now + orbit.reroll_interval.pick_random_value();
                direction.reversed()
            }
            Some(direction) if now >= self.reroll_at => {
                let switched = if rand::rng().random::<f32>() < orbit.switch_chance {
                    direction.reversed()
                } else {
                    direction
                };
                self.roll_distance_offset(orbit);
                self.reroll_at = now + orbit.reroll_interval.pick_random_value();
                switched
            }
            Some(direction) => direction,
        };
        self.direction = Some(direction);

        let tangent = direction.find_tangent(bearing);
        // Positive error means too far out, so the correction points at the target.
        let radial = bearing
            * orbit
                .distance_range
                .center_error(target.distance - self.distance_offset)
            * orbit.radial_gain;
        Some((tangent + radial).normalize_or_zero())
    }
}

impl MotionProgress for OrbitProgress {
    type Tuning = Orbit;
}

/// Circles the target for each running motion tuned by an [`Orbit`].
pub(super) fn apply_orbit(
    time: Res<Time<Virtual>>,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    orbits: Query<&Orbit>,
    mut running_orbits: Query<(
        &MotionOf,
        &RunningMotion,
        &TunedBy,
        &mut OrbitProgress,
        &mut Steering,
    )>,
) {
    for (motion_of, running, tuned_by, mut progress, mut steering) in &mut running_orbits {
        let orbit = orbits.get(tuned_by.source).ok();
        let character = characters.get(motion_of.character).ok();
        let input = character
            .as_ref()
            .map(|character| character.build_steer_input(running.target, &targets, &time));
        *steering = match (orbit, input) {
            (Some(orbit), Some(input)) => {
                let direction = progress.pick_steering_direction(orbit, &input);
                Steering::scaled(direction, 1.0)
            }
            _ => Steering::HALT,
        };
    }
}
