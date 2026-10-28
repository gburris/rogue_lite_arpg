//! Calm ambling inside the anchor.

use bevy::prelude::*;

use super::{Capability, SteerInput};
use crate::character::brain::{BrainRng, Interval, Motion, Tuning};

const WANDER_SPEED_SCALE: f32 = 0.4;

/// Alternates standing and walking one rolled heading while calm.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    pub pause: Interval,
    pub walk: Interval,
    pub speed_scale: f32,
    direction: Vec2,
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
            stretch: None,
        }
    }

    pub fn is_walking(&self) -> bool {
        self.stretch.is_some_and(|stretch| stretch.walking)
    }

    pub(super) fn leave_calm(&mut self) {
        self.stretch = None;
    }
}

/// Makes `Motion::Calm` available only to a character that carries one.
impl Capability for Wander {
    type Key = Motion;

    fn key(&self) -> Motion {
        Motion::Calm
    }

    fn tuning(&self) -> Tuning {
        Tuning::default()
    }
}

pub(super) fn steer(
    input: &SteerInput,
    wander: &mut Wander,
    random: &mut BrainRng,
) -> Option<Vec2> {
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
