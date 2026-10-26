//! Calm ambling inside the anchor.

use bevy::prelude::*;

use super::SteerInput;
use crate::character::brain::{BrainRng, Interval};

const WANDER_SPEED_SCALE: f32 = 0.4;
const JITTER_INTERVAL: f32 = 0.2;

/// Alternates standing and walking one rolled heading while calm. Opt-in at spawn.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    pub pause: Interval,
    pub walk: Interval,
    pub speed_scale: f32,
    direction: Vec2,
    next_direction_at: f32,
    stretch: Option<Stretch>,
}

#[derive(Clone, Copy, Debug)]
struct Stretch {
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
            next_direction_at: 0.0,
            stretch: None,
        }
    }

    pub fn is_walking(&self) -> bool {
        self.stretch.is_some_and(|stretch| stretch.walking)
    }

    pub(super) fn leave_calm(&mut self) {
        self.stretch = None;
    }

    pub(super) fn jitter(&mut self, now: f32, random: &mut BrainRng) -> Vec2 {
        if now >= self.next_direction_at {
            self.direction = random.direction();
            self.next_direction_at = now + JITTER_INTERVAL;
        }
        self.direction
    }
}

pub(super) fn speed_scale(wander: Option<&Wander>) -> f32 {
    wander.map_or(WANDER_SPEED_SCALE, |wander| wander.speed_scale)
}

pub(super) fn steer(
    input: &SteerInput,
    wander: Option<&mut Wander>,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let wander = wander?;
    let stretch = match wander.stretch {
        Some(stretch) if input.now < stretch.until => stretch,
        previous => {
            let walking = previous.is_some_and(|stretch| !stretch.walking);
            let span = if walking { wander.walk } else { wander.pause };
            if walking {
                wander.direction = pick_direction(input, random);
            }
            let stretch = Stretch {
                walking,
                until: input.now + random.range(span.min(), span.max()),
            };
            wander.stretch = Some(stretch);
            stretch
        }
    };
    stretch.walking.then_some(wander.direction)
}

fn pick_direction(input: &SteerInput, random: &mut BrainRng) -> Vec2 {
    let candidate = random.direction();
    let Some(anchor) = input.anchor else {
        return candidate;
    };
    if anchor
        .overshoot_fraction(input.position)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let home = anchor.origin() - input.position;
    if home == Vec2::ZERO {
        return candidate;
    }
    home.normalize_or_zero()
        .rotate(Vec2::from_angle(random.range(-0.6, 0.6)))
}
