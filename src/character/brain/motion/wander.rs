//! Ambling inside the home area.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;

use super::{SteerInput, SteeredCharacter, Steering};
use crate::character::brain::{Interval, Motion};

const WANDER_SPEED_SCALE: f32 = 0.4;

/// Alternates standing and walking one rolled heading while wandering.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    pub pause: Interval,
    pub walk: Interval,
    pub speed_scale: f32,
    direction: Vec2,
    phase: Option<WanderPhase>,
}

#[derive(Clone, Copy, Debug)]
enum WanderPhase {
    Walking { until: f32 },
    Pausing { until: f32 },
}

impl WanderPhase {
    fn until(self) -> f32 {
        match self {
            Self::Walking { until } | Self::Pausing { until } => until,
        }
    }
}

impl Wander {
    pub fn new(pause: Interval, walk: Interval) -> Self {
        Self {
            pause,
            walk,
            speed_scale: WANDER_SPEED_SCALE,
            direction: Vec2::ZERO,
            phase: None,
        }
    }

    pub fn is_walking(&self) -> bool {
        matches!(self.phase, Some(WanderPhase::Walking { .. }))
    }

    fn leave_wander(&mut self) {
        self.phase = None;
    }

    fn pick_steering_direction(&mut self, input: &SteerInput) -> Option<Vec2> {
        let phase = match self.phase {
            Some(phase) if input.now < phase.until() => phase,
            previous => {
                let phase = if matches!(previous, Some(WanderPhase::Pausing { .. })) {
                    self.direction = pick_direction(input);
                    WanderPhase::Walking {
                        until: input.now + self.walk.pick_random_value(),
                    }
                } else {
                    WanderPhase::Pausing {
                        until: input.now + self.pause.pick_random_value(),
                    }
                };
                self.phase = Some(phase);
                phase
            }
        };
        match phase {
            WanderPhase::Walking { .. } => Some(self.direction),
            WanderPhase::Pausing { .. } => None,
        }
    }
}

/// Ambles inside the home area while the brain holds [`Motion::Wander`].
pub(super) fn apply_wander(
    time: Res<Time<Virtual>>,
    mut wanderers: Query<(SteeredCharacter, &mut Wander, &mut Steering)>,
) {
    for (character, mut wander, mut steering) in &mut wanderers {
        if character.brain.motion() != Some(Motion::Wander) {
            wander.leave_wander();
            continue;
        }
        *steering =
            character
                .build_steer_input(Motion::Wander, &time)
                .map_or(Steering::HALT, |input| {
                    let direction = wander.pick_steering_direction(&input);
                    Steering::scaled(direction, wander.speed_scale)
                });
    }
}

fn pick_direction(input: &SteerInput) -> Vec2 {
    let candidate = Vec2::from_angle(rand::rng().random_range(0.0..TAU));
    let Some(home_area) = input.home_area else {
        return candidate;
    };
    if home_area
        .overshoot_fraction(input.position)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let Some(direction_home) = home_area.direction_home(input.position) else {
        return candidate;
    };
    direction_home.rotate(Vec2::from_angle(rand::rng().random_range(-0.6..0.6)))
}
