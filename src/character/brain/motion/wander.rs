//! Ambling inside the home area.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;

use super::{SteerInput, SteeredCharacter, Steering};
use crate::character::brain::{Capability, CapabilityProfile, Interval, Motion};

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
struct WanderPhase {
    walking: bool,
    until: f32,
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
        self.phase.is_some_and(|phase| phase.walking)
    }

    fn leave_wander(&mut self) {
        self.phase = None;
    }
}

/// Makes `Motion::Wander` available only to a character that carries one.
impl Capability for Wander {
    type Key = Motion;

    fn key(&self) -> Motion {
        Motion::Wander
    }

    fn capability_profile(&self) -> CapabilityProfile {
        CapabilityProfile::default()
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
                    let direction = steer(&input, &mut wander);
                    Steering::scaled(direction, wander.speed_scale)
                });
    }
}

fn steer(input: &SteerInput, wander: &mut Wander) -> Option<Vec2> {
    let phase = match wander.phase {
        Some(phase) if input.now < phase.until => phase,
        previous => {
            let walking = previous.is_some_and(|phase| !phase.walking);
            let span = if walking { wander.walk } else { wander.pause };
            if walking {
                wander.direction = pick_direction(input);
            }
            let phase = WanderPhase {
                walking,
                until: input.now + rand::rng().random_range(span.min()..span.max()),
            };
            wander.phase = Some(phase);
            phase
        }
    };
    phase.walking.then_some(wander.direction)
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
    let home = home_area.origin() - input.position;
    if home == Vec2::ZERO {
        return candidate;
    }
    home.normalize_or_zero()
        .rotate(Vec2::from_angle(rand::rng().random_range(-0.6..0.6)))
}
